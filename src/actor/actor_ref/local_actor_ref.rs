use crate::actor::actor::ActorError;
use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_cell::ActorCellBehavior;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::local_actor_ref_ref::LocalActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior, UntypedActorRefBehavior};
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::props::Props;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::message::Message;
use crate::dispatch::system_message::SystemMessage;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LocalActorRef<Msg: Message> {
  path: ActorPath,
  actor_cell: Option<ActorCellRef<Msg>>,
}

unsafe impl<Msg: Message> Send for LocalActorRef<Msg> {}
unsafe impl<Msg: Message> Sync for LocalActorRef<Msg> {}

impl<Msg: Message> PartialEq for LocalActorRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl<Msg: Message> UntypedActorRefBehavior for LocalActorRef<Msg> {
  fn tell_any(&mut self, msg: AnyMessage) {
    let msg_ = msg.take::<Msg>().unwrap();
    self.tell(msg_);
  }
}

impl<Msg: Message> LocalActorRef<Msg> {
  pub fn new(path: ActorPath) -> Self {
    Self { path, actor_cell: None }
  }

  pub fn initialize(
    &mut self,
    actor_system_context: ActorSystemContext,
    mailbox_type: MailboxType,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) {
    let actor_ref_ref = self.clone().to_local_actor_ref_ref().to_actor_ref().to_actor_ref_ref();
    let mut actor_cell = ActorCellRef::new(actor_system_context, actor_ref_ref, props, parent_ref);
    actor_cell.initialize(true, mailbox_type);
    self.actor_cell = Some(actor_cell);
  }

  pub fn to_local_actor_ref_ref(self) -> LocalActorRefRef<Msg> {
    LocalActorRefRef::new(self)
  }
}

impl<Msg: Message> ActorRefBehavior for LocalActorRef<Msg> {
  type M = Msg;

  fn path(&self) -> ActorPath {
    self.path.clone()
  }

  fn tell(&mut self, msg: Self::M) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    let envelop = Envelope::new(msg);
    actor_cell.send_message(envelop);
  }
}

impl<Msg: Message> InternalActorRefBehavior for LocalActorRef<Msg> {
  type Msg = Msg;

  fn actor_cell_ref(&self) -> ActorCellRef<Self::Msg> {
    let actor_cell = self.actor_cell.as_ref().unwrap();
    actor_cell.clone()
  }

  fn actor_system_context(&self) -> ActorSystemContext {
    let actor_cell = self.actor_cell.as_ref().unwrap();
    actor_cell.actor_system_context()
  }

  fn mailbox_type(&self) -> MailboxType {
    let actor_cell = self.actor_cell.as_ref().unwrap();
    actor_cell.mailbox_type()
  }

  fn start(&mut self) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.start();
  }

  fn resume(&mut self, caused_by_failure: ActorError) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.resume(caused_by_failure);
  }

  fn suspend(&mut self) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.suspend();
  }

  fn stop(&mut self) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.stop();
  }

  fn restart(&mut self, cause: ActorError) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.restart(cause);
  }

  fn send_system_message(&mut self, message: &mut SystemMessage) {
    let actor_cell = self.actor_cell.as_mut().unwrap();
    actor_cell.send_system_message(message);
  }
}
