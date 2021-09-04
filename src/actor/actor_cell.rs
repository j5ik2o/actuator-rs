use std::sync::{Arc, Mutex};

use crate::actor::actor_context::ActorContext;
use crate::actor::actor_ref::{
  ActorRef, InternalActorRef, InternalActorRefArc, ToUntypedActorRef, UntypedActorRef,
  UntypedActorRefArc,
};
use crate::actor::actor_ref_factory::ActorRefFactory;
use crate::actor::actor_ref_provider::{ActorRefProviderArc};
use crate::actor::cell::Cell;
use crate::actor::children::Children;
use crate::actor::ExtendedCell;
use crate::actor_system::{ActorSystem, ActorSystemArc};
use crate::kernel::any_message_sender::{AnyMessageSenderArc};

use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<Mutex<ActorCellInner>>,
}

#[derive(Debug, Clone)]
struct ActorCellInner {
  system: ActorSystemArc,
  self_ref: Option<InternalActorRefArc>,
  // parent_ref: Arc<dyn InternalActorRef>,
  // path: ActorPath,
  children: Children,
  // mailbox: Arc<dyn AnyMessageSender>,
  // system_mailbox: MailboxSender<SystemMessage>,
}

impl ActorCell {
  pub fn to_extended_cell<M: Message>(self, mailbox_sender: MailboxSender<M>) -> ExtendedCell<M> {
    ExtendedCell::from_actor_cell(self, mailbox_sender)
  }

  pub fn set_self_ref(&mut self, self_ref: InternalActorRefArc) {
    let mut inner = self.inner.lock().unwrap();
    inner.self_ref = Some(self_ref);
  }

  pub fn new(
    system: ActorSystemArc,
    // parent_ref: Arc<dyn InternalActorRef>,
    // path: ActorPath,
    //   mailbox: Arc<dyn AnyMessageSender>,
    //   system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      inner: Arc::from(Mutex::new(ActorCellInner {
        system,
        self_ref: None,
        // parent_ref,
        // path,
        children: Children::new(),
        //      mailbox,
        //      system_mailbox,
      })),
    }
  }

  // pub fn path(&self) -> &ActorPath {
  //   &self.inner.path
  // }

  pub fn mailbox(&self) -> AnyMessageSenderArc {
    todo!()
    // self.inner.mailbox.clone()
  }

  pub fn my_self(&self) -> Option<InternalActorRefArc> {
    let inner = self.inner.lock().unwrap();
    inner.self_ref.clone()
  }

  // pub fn mailbox_for_system(&self) -> MailboxSender<SystemMessage> {
  //   self.inner.system_mailbox.clone()
  // }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}

impl ActorRefFactory for ActorCell {
  fn system(&self) -> ActorSystemArc {
    let inner = self.inner.lock().unwrap();
    inner.system.clone()
  }

  fn provider(&self) -> ActorRefProviderArc {
    let inner = self.inner.lock().unwrap();
    inner.system.provider()
  }

  fn guardian(&self) -> InternalActorRefArc {
    todo!()
  }

  fn lookup_root(&self) -> InternalActorRefArc {
    todo!()
  }

  fn actor_of(&self) -> Arc<dyn ActorRef> {
    todo!()
  }

  fn stop(&self, _actor_ref: Arc<dyn ActorRef>) {
    todo!()
  }
}

impl ActorContext for ActorCell {
  fn self_ref(&self) -> UntypedActorRefArc {
    let inner = self.inner.lock().unwrap();
    inner
      .self_ref
      .as_ref()
      .unwrap()
      .clone()
      .to_untyped_actor_ref()
  }

  fn parent_ref(&self) -> UntypedActorRefArc {
    todo!()
    //    self.inner.parent_ref.clone().to_untyped_actor_ref()
  }

  fn children(&self) -> Children {
    let inner = self.inner.lock().unwrap();
    inner.children.clone()
  }

  fn child(&self, name: &str) -> Option<Box<dyn UntypedActorRef>> {
    let inner = self.inner.lock().unwrap();
    let x = inner
      .children
      .iter()
      .find(|e| e.name() == name)
      .map(|e| Box::new(e) as Box<dyn UntypedActorRef>);
    x
  }

  fn system(&self) -> Arc<dyn ActorSystem> {
    let inner = self.inner.lock().unwrap();
    inner.system.clone()
  }
}

impl Cell for ActorCell {
  fn system(&self) -> Arc<dyn ActorSystem> {
    let inner = self.inner.lock().unwrap();
    inner.system.clone()
  }

  fn start(&self) -> Arc<dyn ActorContext> {
    todo!()
  }

  fn suspend(&self) {
    todo!()
  }

  fn resume(_panic_by_failure: &str) {
    todo!()
  }

  fn restart(_panic_message: &str) {
    todo!()
  }

  fn stop(&self) {
    todo!()
  }

  fn parent(&self) -> Arc<dyn InternalActorRef> {
    todo!()
    //    self.inner.parent_ref.clone()
  }
}
