use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::actor::actor::{Actor, ActorError, ActorMutableBehavior};
use crate::actor::actor_cell::ActorCellBehavior;
use crate::actor::actor_context::ActorContextRef;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior};
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::child_state::ChildState;
use crate::actor::children::{Children, ChildrenBehavior};
use crate::actor::props::Props;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::dead_letter_mailbox::DeadLetterMailbox;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox::Mailbox;

use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::mailbox::{MailboxBehavior, MailboxInternal};
use crate::dispatch::message::Message;
use crate::dispatch::message_dispatcher::{MessageDispatcherBehavior, MessageDispatcherRef};
use crate::dispatch::message_queue::MessageQueueSize;
use crate::dispatch::system_message::{SystemMessage, SystemMessageQueueBehavior};

#[derive(Debug, Clone)]
struct ActorCell<Msg: Message> {
  actor_system_context: ActorSystemContext,
  actor_ref: ActorRefRef<Msg>,
  props: Rc<dyn Props<Msg>>,
  actor: Option<Rc<RefCell<Actor<Msg>>>>,
  mailbox: Option<Mailbox<Msg>>,
  children: Children<AnyMessage>,
  parent_ref: Option<ActorRefRef<AnyMessage>>,
}

impl<Msg: Message> ActorCell<Msg> {
  fn new(
    actor_system_context: ActorSystemContext,
    actor_ref: ActorRefRef<Msg>,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) -> Self {
    Self {
      actor_system_context,
      actor_ref,
      props,
      actor: None,
      mailbox: None,
      children: Children::new(),
      parent_ref,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ActorCellRef<Msg: Message> {
  inner: Rc<RefCell<ActorCell<Msg>>>,
}

unsafe impl<Msg: Message> Send for ActorCellRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorCellRef<Msg> {}

impl<Msg: Message> PartialEq for ActorCellRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.inner, &other.inner)
  }
}

impl<Msg: Message> ActorCellRef<Msg> {
  pub fn new(
    actor_system_context: ActorSystemContext,
    actor_ref: ActorRefRef<Msg>,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) -> Self {
    Self::new_with_inner(Rc::new(RefCell::new(ActorCell::new(
      actor_system_context,
      actor_ref,
      props,
      parent_ref,
    ))))
  }

  fn new_with_inner(inner: Rc<RefCell<ActorCell<Msg>>>) -> Self {
    Self { inner }
  }

  pub fn initialize(&mut self, send_supervise: bool, mailbox_type: MailboxType) -> Self {
    {
      let inner = self.inner.borrow_mut();
      match &inner.mailbox {
        Some(mailbox) => {
          if mailbox.is_closed() {
            panic!("mailbox is closed");
          }
        }
        _ => {}
      }
    }

    let mut new_mailbox_ref = {
      let inner = self.inner.borrow();
      let mailbox = inner
        .actor_system_context
        .dispatcher_ref
        .create_mailbox(self.clone(), mailbox_type.clone());
      mailbox
    };

    self.swap_mailbox(Some(&mut new_mailbox_ref));

    let mut inner = self.inner.borrow_mut();
    let actor_ref = inner.actor_ref.clone();
    let mailbox_ref = inner.mailbox.as_mut().unwrap();
    mailbox_ref.set_actor_cell(self.clone());
    mailbox_ref.system_enqueue(actor_ref, &mut SystemMessage::of_create(None));

    if send_supervise {
      if let Some(parent) = &mut inner.parent_ref {
        parent
          .as_local()
          .unwrap()
          .send_system_message(&mut SystemMessage::of_supervise(inner.actor_ref.to_any_ref_ref(), true));
      }
    }

    self.clone()
  }

  pub fn actor(&self) -> Option<Rc<RefCell<Actor<Msg>>>> {
    let inner = self.inner.borrow();
    inner.actor.clone()
  }

  pub fn actor_system_context(&self) -> ActorSystemContext {
    let inner = self.inner.borrow();
    inner.actor_system_context.clone()
  }

  pub fn mailbox_type(&self) -> MailboxType {
    let inner = self.inner.borrow();
    inner.mailbox.as_ref().unwrap().mailbox_type()
  }

  pub fn actor_ref_ref(&self) -> ActorRefRef<Msg> {
    let inner = self.inner.borrow();
    inner.actor_ref.clone()
  }

  pub fn dispatcher(&self) -> MessageDispatcherRef {
    let inner = self.inner.borrow();
    inner.actor_system_context.dispatcher_ref.clone()
  }

  pub fn mailbox(&self) -> Mailbox<Msg> {
    let inner = self.inner.borrow();
    inner.mailbox.as_ref().unwrap().clone()
  }

  pub fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>> {
    let inner = self.inner.borrow();
    let mailboxes = inner.actor_system_context.dispatcher_ref.mailboxes();
    let mailboxes_guard = mailboxes.lock().unwrap();
    mailboxes_guard.dead_letter_mailbox()
  }

  fn is_terminated(&self) -> bool {
    let inner = self.inner.borrow();
    let mailbox_ref = inner.mailbox.as_ref().unwrap();
    mailbox_ref.is_closed()
  }

  fn swap_mailbox(&mut self, mailbox: Option<&mut Mailbox<Msg>>) {
    let mut inner = self.inner.borrow_mut();
    match (inner.mailbox.as_mut(), mailbox) {
      (Some(mf), Some(mb)) => {
        if mf == mb {
          return;
        } else {
          let mailbox_ref = inner.mailbox.as_mut().unwrap();
          std::mem::swap(&mut *mailbox_ref, mb);
        }
      }
      (Some(_), None) => {
        inner.mailbox = None;
      }
      (None, Some(mb)) => {
        inner.mailbox = Some(mb.clone());
      }
      (None, None) => {
        return;
      }
    }
  }

  pub(crate) fn new_actor_ref<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRefRef<U> {
    let inner = { self.inner.borrow().clone() };
    let actor_path = ActorPath::of_child(inner.actor_ref.path(), name.to_string(), 0);
    let parent_ref = inner.parent_ref;

    let mut actor_ref = ActorRefRef::of_local(actor_path);

    actor_ref.initialize(
      inner.actor_system_context.clone(),
      MailboxType::Unbounded,
      props,
      parent_ref,
    );

    actor_ref
  }

  pub(crate) fn init_child<U: Message>(&mut self, actor_ref_ref: ActorRefRef<U>) -> Option<ChildState<AnyMessage>> {
    let mut inner = self.inner.borrow_mut();
    let wrapper = actor_ref_ref.to_any_ref_ref();
    inner.children.init_child(wrapper)
  }
}

impl<Msg: Message> ActorCellBehavior<Msg> for ActorCellRef<Msg> {
  fn start(&mut self) {
    log::debug!("ActorCellRef::suspend: start");
    let mut inner = self.inner.borrow_mut();
    inner.actor_system_context.dispatcher_ref.attach(self.clone());
    log::debug!("ActorCellRef::suspend: finished");
  }

  fn suspend(&mut self) {
    log::debug!("ActorCellRef::suspend: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_suspend());
    log::debug!("ActorCellRef::suspend: finished");
  }

  fn resume(&mut self, caused_by_failure: ActorError) {
    log::debug!("ActorCellRef::resume: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_resume(caused_by_failure));
    log::debug!("ActorCellRef::resume: finished");
  }

  fn restart(&mut self, cause: ActorError) {
    log::debug!("ActorCellRef::restart: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_recreate(cause));
    log::debug!("ActorCellRef::restart: finished");
  }

  fn stop(&mut self) {
    log::debug!("ActorCellRef::stop: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_terminate());
    log::debug!("ActorCellRef::stop: finished");
  }

  fn has_messages(&self) -> bool {
    log::debug!("ActorCellRef::has_messages: start");
    let inner = self.inner.borrow();
    let mailbox_guard = inner.mailbox.as_ref().unwrap();
    let result = mailbox_guard.has_messages();
    log::debug!("ActorCellRef::has_messages: finished");
    result
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    log::debug!("ActorCellRef::number_of_messages: start");
    let inner = self.inner.borrow();
    let mailbox_guard = inner.mailbox.as_ref().unwrap();
    let result = mailbox_guard.number_of_messages();
    log::debug!("ActorCellRef::number_of_messages: finished");
    result
  }

  fn send_message(&mut self, msg: Envelope) {
    log::debug!("ActorCellRef::send_message: start");
    let mut dispatcher = {
      let inner = self.inner.borrow_mut();
      inner.actor_system_context.dispatcher_ref.clone()
    };
    dispatcher.dispatch(self.clone(), msg);
    log::debug!("ActorCellRef::send_message: finished");
  }

  fn send_system_message(&mut self, msg: &mut SystemMessage) {
    log::debug!("ActorCellRef::send_system_message: start");
    let mut dispatcher = {
      let inner = self.inner.borrow_mut();
      inner.actor_system_context.dispatcher_ref.clone()
    };
    dispatcher.system_dispatch(self.clone(), msg);
    log::debug!("ActorCellRef::send_system_message: finished");
  }

  fn invoke(&mut self, msg: Envelope) {
    log::debug!("ActorCellRef::invoke: start");
    let mut inner = self.inner.borrow_mut();
    let ctx = ActorContextRef::new(self.clone());
    let mut actor = inner.actor.as_mut().unwrap().borrow_mut();
    actor.receive(ctx, msg.typed_message());
    log::debug!("ActorCellRef::invoke: finished");
  }

  fn system_invoke(&mut self, msg: &SystemMessage) {
    fn init_arc<Msg: Message>(s: &mut ActorCellRef<Msg>) -> Rc<RefCell<Actor<Msg>>> {
      let mut inner = s.inner.borrow_mut();
      let new_actor = inner.props.new_actor();
      inner.actor = Some(Rc::new(RefCell::new(new_actor)));
      inner.actor.as_ref().unwrap().clone()
    }
    fn get_arc<Msg: Message>(s: &mut ActorCellRef<Msg>) -> Rc<RefCell<Actor<Msg>>> {
      let inner = s.inner.borrow();
      inner.actor.as_ref().unwrap().clone()
    }
    log::debug!("ActorCellRef::system_invoke: start");
    match msg {
      SystemMessage::Create { .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Creating actor: actor_path = {}", inner.actor_ref.path());
        }
        let actor_rc = init_arc(self);
        let ctx = ActorContextRef::new(self.clone());
        let mut actor = actor_rc.borrow_mut();
        actor.pre_start(ctx);
      }
      SystemMessage::Recreate { cause, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Recreating actor: actor_path = {}", inner.actor_ref.path());
        }
        let actor_rc = get_arc(self);
        let ctx = ActorContextRef::new(self.clone());
        let mut actor = actor_rc.borrow_mut();
        actor.pre_restart(ctx, cause.clone());
      }
      SystemMessage::Suspend { .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Suspending actor: actor_path = {}", inner.actor_ref.path());
        }
        let actor_rc = get_arc(self);
        let ctx = ActorContextRef::new(self.clone());
        let mut actor = actor_rc.borrow_mut();
        actor.pre_suspend(ctx);
      }
      SystemMessage::Resume { caused_by_failure, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Resuming actor: actor_path = {}", inner.actor_ref.path());
        }
        let actor_rc = get_arc(self);
        let ctx = ActorContextRef::new(self.clone());
        let mut actor = actor_rc.borrow_mut();
        actor.post_resume(ctx, caused_by_failure.clone());
      }
      SystemMessage::Terminate { .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!(
            "Terminating actor: actor_path = {}",
            inner.actor_ref.as_local().unwrap().path()
          );
        }
        let actor_rc = get_arc(self);
        let ctx = ActorContextRef::new(self.clone());
        let mut actor = actor_rc.borrow_mut();
        actor.post_stop(ctx);
      }
      SystemMessage::Supervise { child, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!(
            "Supervising actor: actor_path = {}",
            inner.actor_ref.as_local().unwrap().path()
          );
        }
        if !self.is_terminated() {
          match self.init_child(child.clone()) {
            Some(_) => {
              // TODO: handle_supervise
            }
            None => {}
          }
        }
      }
      _ => {}
    }
    log::debug!("ActorCellRef::system_invoke: finished");
  }
}
