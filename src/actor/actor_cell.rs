use std::sync::Arc;

use crate::actor::actor_context::ActorContext;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef, ToUntypedActorRef, UntypedActorRef};
use crate::actor::actor_ref_factory::ActorRefFactory;
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor::cell::Cell;
use crate::actor_system::ActorSystem;
use crate::kernel::any_message_sender::AnyMessageSender;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::message::Message;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Debug, Clone)]
struct ActorCellInner {
  system: Arc<dyn ActorSystem>,
  self_ref: Arc<dyn InternalActorRef>,
  parent_ref: Arc<dyn InternalActorRef>,
  path: ActorPath,
  mailbox: Arc<dyn AnyMessageSender>,
  system_mailbox: MailboxSender<SystemMessage>,
}

impl ActorRefFactory for ActorCell {
  fn system(&self) -> Arc<dyn ActorSystem> {
    self.inner.system.clone()
  }

  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    self.inner.system.provider()
  }

  fn guardian(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn lookup_root(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn actor_of(&self) -> Arc<dyn ActorRef> {
    todo!()
  }

  fn stop(&self, actor_ref: Arc<dyn ActorRef>) {
    todo!()
  }
}

impl ActorContext for ActorCell {
  fn self_ref(&self) -> Arc<dyn UntypedActorRef> {
    self.inner.self_ref.clone().to_untyped_actor_ref()
  }

  fn parent_ref(&self) -> Arc<dyn UntypedActorRef> {
    self.inner.parent_ref.clone().to_actor_ref()
  }

  fn children(&self) -> Vec<Arc<dyn UntypedActorRef>> {
    todo!()
  }

  fn child(&self, name: &str) -> Option<Arc<dyn UntypedActorRef>> {
    todo!()
  }

  fn system(&self) -> Arc<dyn ActorSystem> {
    self.inner.system.clone()
  }
}

impl Cell for ActorCell {
  fn system(&self) -> Arc<dyn ActorSystem> {
    self.inner.system.clone()
  }

  fn start(&self) -> Arc<dyn ActorContext> {
    todo!()
  }

  fn suspend(&self) {
    todo!()
  }

  fn resume(panic_by_failure: &str) {
    todo!()
  }

  fn restart(panic_message: &str) {
    todo!()
  }

  fn stop(&self) {
    todo!()
  }

  fn parent(&self) -> Arc<dyn InternalActorRef> {
    self.inner.parent_ref.clone()
  }
}

impl ActorCell {
  pub fn new(
    system: Arc<dyn ActorSystem>,
    self_ref: Arc<dyn InternalActorRef>,
    parent_ref: Arc<dyn InternalActorRef>,
    path: ActorPath,
    mailbox: Arc<dyn AnyMessageSender>,
    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      inner: Arc::from(ActorCellInner {
        system,
        self_ref,
        parent_ref,
        path,
        mailbox,
        system_mailbox,
      }),
    }
  }

  pub fn path(&self) -> &ActorPath {
    &self.inner.path
  }

  pub fn mailbox(&self) -> Arc<dyn AnyMessageSender> {
    self.inner.mailbox.clone()
  }

  pub fn mailbox_for_system(&self) -> MailboxSender<SystemMessage> {
    self.inner.system_mailbox.clone()
  }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}
