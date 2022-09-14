use crate::actor::actor::ActorError;
use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_path::ActorPathBehavior;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior};
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::message::Message;
use crate::dispatch::message_dispatcher::MessageDispatcherRef;
use crate::dispatch::system_message::SystemMessage;
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
  _phantom_data: std::marker::PhantomData<Msg>,
  failed: Rc<RefCell<FailedInfo>>,
  dispatcher_ref: MessageDispatcherRef,
  self_ref: ActorRefRef<Msg>,
  parent_ref: ActorRefRef<AnyMessage>,
  current_message: Rc<RefCell<Option<Envelope>>>,
}

impl<Msg: Message> FaultHandling<Msg> {
  pub fn new(
    self_ref: ActorRefRef<Msg>,
    parent_ref: ActorRefRef<AnyMessage>,
    dispatcher_ref: MessageDispatcherRef,
    current_message: Rc<RefCell<Option<Envelope>>>,
  ) -> Self {
    Self {
      _phantom_data: std::marker::PhantomData,
      failed: Rc::new(RefCell::new(FailedInfo::NoFailedInfo)),
      dispatcher_ref,
      self_ref,
      parent_ref,
      current_message,
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

  pub fn handle_invoke_failure<U: Message>(
    &mut self,
    actor_cell: ActorCellRef<U>,
    child_not_suspend: Vec<ActorRefRef<U>>,
    error: ActorError,
  ) {
    if !self.is_failed() {
      self.suspend_non_recursive(actor_cell);
      let cm = self.current_message.borrow().clone();
      match cm {
        Some(envelope) => {
          let skip = match &envelope.typed_message() {
            SystemMessage::Failed { .. } => {
              let sender = envelope.sender().as_ref().unwrap().clone();
              self.set_failed(sender.clone());
              let mut set = HashSet::new();
              set.insert(sender);
              set
            }
            _ => {
              self.set_failed(self.self_ref.to_any_ref_ref().clone());
              HashSet::new()
            }
          };
          // suspend_children(skip ++ child_not_suspend)
          self
            .parent_ref
            .as_local()
            .as_mut()
            .unwrap()
            .send_system_message(&mut SystemMessage::of_failed(
              self.self_ref.to_any_ref_ref().clone(),
              error,
              self.self_ref.path().uid(),
            ));
        }
        None => {}
      }
    }
  }

  fn suspend_non_recursive<U: Message>(&mut self, actor_cell: ActorCellRef<U>) {
    // self.dispatcher_ref.suspend(actor_cell);
  }

  fn resume_non_recursive<U: Message>(&mut self, actor_cell: ActorCellRef<U>) {
    // self.dispatcher_ref.resume(actor_cell);
  }
}
