use crate::actor::actor::ActorError;
use std::cell::RefCell;
use std::rc::Rc;

use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref::ActorRef;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::local_actor_ref::LocalActorRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior, UntypedActorRefBehavior};
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::props::Props;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::message::Message;
use crate::dispatch::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct LocalActorRefRef<Msg: Message> {
  inner: Rc<RefCell<LocalActorRef<Msg>>>,
}

impl<Msg: Message> PartialEq for LocalActorRefRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.borrow();
    let r = other.inner.borrow();
    &*l == &*r
  }
}

impl<Msg: Message> LocalActorRefRef<Msg> {
  pub fn new(inner: LocalActorRef<Msg>) -> Self {
    Self {
      inner: Rc::new(RefCell::new(inner)),
    }
  }

  pub fn new_with(path: ActorPath) -> Self {
    Self::new(LocalActorRef::new(path))
  }

  pub fn initialize(
    &mut self,
    self_ref: ActorRefRef<Msg>,
    actor_system_context: ActorSystemContext,
    mailbox_type: MailboxType,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) {
    let mut inner = self.inner.borrow_mut();
    inner.initialize(self_ref, actor_system_context, mailbox_type, props, parent_ref);
  }

  pub fn to_arc_mutex(&self) -> Rc<RefCell<LocalActorRef<Msg>>> {
    self.inner.clone()
  }

  pub fn to_actor_ref(self) -> ActorRef<Msg> {
    ActorRef::Local(self)
  }
}

impl<Msg: Message> UntypedActorRefBehavior for LocalActorRefRef<Msg> {
  fn tell_any(&mut self, msg: AnyMessage) {
    let mut inner = self.inner.borrow_mut();
    inner.tell_any(msg)
  }
}

impl<Msg: Message> ActorRefBehavior for LocalActorRefRef<Msg> {
  type M = Msg;

  fn path(&self) -> ActorPath {
    let inner = self.inner.borrow();
    inner.path()
  }

  fn tell(&mut self, msg: Self::M) {
    let mut inner = self.inner.borrow_mut();
    inner.tell(msg)
  }
}

impl<Msg: Message> InternalActorRefBehavior for LocalActorRefRef<Msg> {
  type Msg = Msg;

  fn actor_cell_ref(&self) -> ActorCellRef<Self::Msg> {
    let inner = self.inner.borrow();
    inner.actor_cell_ref()
  }

  fn actor_system_context(&self) -> ActorSystemContext {
    let inner = self.inner.borrow();
    inner.actor_system_context()
  }

  fn mailbox_type(&self) -> MailboxType {
    let inner = self.inner.borrow();
    inner.mailbox_type()
  }

  fn start(&mut self) {
    let mut inner = self.inner.borrow_mut();
    inner.start();
  }

  fn resume(&mut self, caused_by_failure: Option<ActorError>) {
    let mut inner = self.inner.borrow_mut();
    inner.resume(caused_by_failure);
  }

  fn suspend(&mut self) {
    let mut inner = self.inner.borrow_mut();
    inner.suspend();
  }

  fn stop(&mut self) {
    let mut inner = self.inner.borrow_mut();
    inner.stop();
  }

  fn restart(&mut self, cause: ActorError) {
    let mut inner = self.inner.borrow_mut();
    inner.restart(cause);
  }

  fn send_system_message(&mut self, message: &mut SystemMessage) {
    let mut inner = self.inner.borrow_mut();
    inner.send_system_message(message);
  }
}
