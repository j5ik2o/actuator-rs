use crate::actor::actor::{ActorError, ActorMutableBehavior, ActorResult};
use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_cell::children_container::SuspendReason;
use crate::actor::actor_path::ActorPathBehavior;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior};
use crate::dispatch::any_message::AnyMessage;

use crate::dispatch::mailbox::MailboxInternal;
use crate::dispatch::message::Message;

use crate::actor::actor_cell::{ActorCellBehavior, ActorCellInnerBehavior};
use crate::actor::child_state::ChildState;
use crate::dispatch::system_message::SystemMessage;
use crate::ActuatorError;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum FailedInfo {
  NoFailedInfo,
  FailedRef(ActorRefRef<AnyMessage>),
  FailedFatally,
}

pub trait FaultHandlingBehavior {
  type Msg: Message;
  fn suspend_non_recursive(&mut self, actor_cell: ActorCellRef<Self::Msg>);
  fn resume_non_recursive(&mut self, actor_cell: ActorCellRef<Self::Msg>);
}

#[derive(Debug, Clone)]
pub struct FaultHandling<Msg: Message> {
  failed: Rc<RefCell<FailedInfo>>,
  actor_cell_ref: ActorCellRef<Msg>,
}

impl<Msg: Message> FaultHandling<Msg> {
  pub fn new(actor_cell_ref: ActorCellRef<Msg>) -> Self {
    Self {
      failed: Rc::new(RefCell::new(FailedInfo::NoFailedInfo)),
      actor_cell_ref,
    }
  }

  pub fn is_failed(&self) -> bool {
    let failed = self.failed.borrow();
    match &*failed {
      FailedInfo::FailedRef(..) => true,
      _ => false,
    }
  }

  pub fn is_failed_fatally(&self) -> bool {
    let failed = self.failed.borrow();
    match &*failed {
      FailedInfo::FailedFatally => true,
      _ => false,
    }
  }

  pub fn perpetrator(&self) -> Option<ActorRefRef<AnyMessage>> {
    let failed = self.failed.borrow();
    match &*failed {
      FailedInfo::FailedRef(perpetrator) => Some(perpetrator.clone()),
      _ => None,
    }
  }

  pub fn set_failed(&mut self, perpetrator: ActorRefRef<AnyMessage>) {
    let mut failed = self.failed.borrow_mut();
    let new_filed = match &*failed {
      FailedInfo::FailedFatally => FailedInfo::FailedFatally,
      _ => FailedInfo::FailedRef(perpetrator),
    };
    *failed = new_filed;
  }

  pub fn clear_failed(&mut self) {
    let mut failed = self.failed.borrow_mut();
    let new_filed = match &*failed {
      FailedInfo::FailedRef(..) => FailedInfo::NoFailedInfo,
      other => other.clone(),
    };
    *failed = new_filed;
  }

  pub fn set_failed_fatally(&mut self) {
    let mut failed = self.failed.borrow_mut();
    *failed = FailedInfo::FailedFatally;
  }

  pub fn create(&mut self, self_ref: ActorRefRef<Msg>, failure: Option<ActuatorError>) -> ActorResult<()> {
    failure.iter().for_each(|e| panic!("create failed: {}", e));
    let created_rc = self.actor_cell_ref.new_actor();
    let ctx = self.actor_cell_ref.new_actor_context(self_ref);
    {
      let mut created = created_rc.borrow_mut();
      match created.around_pre_start(ctx) {
        Ok(_) => {}
        e @ Err(_) => {
          self.actor_cell_ref.set_current_message(None);
          self.set_failed_fatally();
          self.actor_cell_ref.clear_actor();
          return e;
        }
      }
    }
    let rt = self.actor_cell_ref.actor_system_context().runtime.clone();
    self.actor_cell_ref.check_receive_timeout(self_ref, rt, true);
    Ok(())
  }

  pub(crate) fn fault_recreate(&mut self, self_ref: ActorRefRef<Msg>, cause: ActorError) {
    if self.actor_cell_ref.actor().is_none() {
      self.fault_create(self_ref);
    } else if self.actor_cell_ref.children().is_normal() {
      if let Some(actor_rc) = self.actor_cell_ref.actor() {
        if self.is_failed_fatally() {
          let mut actor = actor_rc.borrow_mut();
          let ctx = self.actor_cell_ref.new_actor_context(self_ref);
          let optional_message = self.actor_cell_ref.current_message().borrow().clone();
          let _result =
            actor.around_pre_restart(ctx, cause.clone(), optional_message.map(|e| e.typed_message().unwrap()));
        }
      }
      // clearActorFields
      self.actor_cell_ref.set_current_message(None);
      if !self
        .actor_cell_ref
        .children()
        .set_children_termination_reason(SuspendReason::Recreation { cause })
      {
        self.finish_recreate();
      }
    } else {
      self.fault_resume(self_ref, None);
    }
  }

  pub fn fault_suspend(&mut self) {
    self.suspend_non_recursive();
    self.suspend_children(HashSet::new());
  }

  pub fn suspend_children(&mut self, except_for: HashSet<ActorRefRef<AnyMessage>>) {
    self
      .actor_cell_ref
      .children()
      .stats()
      .iter()
      .for_each(|child| match child {
        ChildState::ChildRestartStats(stats) if !except_for.contains(&stats.child) => {
          stats.child.as_local().unwrap().suspend()
        }
        _ => {}
      });
  }

  pub fn resume_children(&mut self, cause_by_failure: Option<ActorError>, perpetrator: ActorRefRef<AnyMessage>) {
    self
      .actor_cell_ref
      .children()
      .stats()
      .iter()
      .for_each(|child| match child {
        ChildState::ChildRestartStats(stats) => stats.child.as_local().unwrap().resume(if perpetrator == stats.child {
          cause_by_failure.clone()
        } else {
          None
        }),
        _ => {}
      });
  }

  pub fn fault_resume(&mut self, self_ref: ActorRefRef<Msg>, cause_by_failure: Option<ActorError>) {
    if self.actor_cell_ref.actor().is_none() {
      self.fault_create(self_ref);
    } else if self.is_failed_fatally() && cause_by_failure.is_some() {
      self.fault_recreate(self_ref, cause_by_failure.unwrap());
    } else {
      match self.perpetrator() {
        Some(perp) => {
          self.resume_non_recursive();
          if cause_by_failure.is_some() {
            self.clear_failed();
          }
          self.resume_children(cause_by_failure, perp.to_any_ref_ref())
        }
        _ => {}
      }
    }
  }

  pub fn fault_create(&mut self, self_ref: ActorRefRef<Msg>) {
    assert!(
      self.actor_cell_ref.mailbox().is_suspend(),
      "mailbox must be suspended during failed creation, status={}",
      self.actor_cell_ref.mailbox().get_status()
    );
    assert_eq!(self.perpetrator(), Some(self_ref.to_any_ref_ref()));

    self.actor_cell_ref.cancel_receive_timeout();

    self.actor_cell_ref.children_vec().iter().for_each(|child| {
      child.as_local().unwrap().stop();
    });

    if !self
      .actor_cell_ref
      .children()
      .set_children_termination_reason(SuspendReason::Creation)
    {
      self.finish_create(self_ref);
    }
  }

  fn finish_create(&mut self, self_ref: ActorRefRef<Msg>) {
    self.resume_non_recursive();
    self.clear_failed();
    match self.create(self_ref, None) {
      Ok(_) => {}
      Err(e) => self.handle_invoke_failure(self_ref, Vec::new(), e.clone()),
    }
  }

  fn finish_recreate(&mut self) {}

  pub fn handle_invoke_failure(
    &mut self,
    self_ref: ActorRefRef<Msg>,
    child_not_suspend: Vec<ActorRefRef<AnyMessage>>,
    error: ActorError,
  ) {
    if !self.is_failed() {
      self.suspend_non_recursive();
      let cm = self.actor_cell_ref.current_message().borrow().clone();
      match cm {
        Some(envelope) => {
          let mut skip = match envelope.typed_message::<SystemMessage>() {
            Ok(SystemMessage::Failed { .. }) => {
              let sender = envelope.sender().as_ref().unwrap().clone();
              self.set_failed(sender.clone());
              let mut set = HashSet::new();
              set.insert(sender);
              set
            }
            _ => {
              // TODO
              self.set_failed(self_ref.to_any_ref_ref().clone());
              HashSet::new()
            }
          };
          skip.extend(child_not_suspend);
          self.suspend_children(skip);
          self
            .actor_cell_ref
            .parent_ref()
            .as_mut()
            .unwrap()
            .as_local()
            .as_mut()
            .unwrap()
            .send_system_message(&mut SystemMessage::of_failed(
              self_ref.to_any_ref_ref().clone(),
              error,
              self_ref.path().uid(),
            ));
        }
        None => {}
      }
    }
  }

  fn suspend_non_recursive(&mut self) {
    let dispatcher = self.actor_cell_ref.dispatcher();
    dispatcher.suspend(self.actor_cell_ref.clone());
  }

  fn resume_non_recursive(&mut self) {
    let dispatcher = self.actor_cell_ref.dispatcher();
    dispatcher.resume(self.actor_cell_ref.clone());
  }
}
