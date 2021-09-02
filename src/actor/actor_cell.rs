
use std::sync::Arc;




use crate::actor::actor_path::ActorPath;
use crate::kernel::any_message_sender::AnyMessageSender;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::system_message::SystemMessage;
use crate::actor_system::ActorSystem;
use crate::actor::actor_context::ActorContext;
use crate::kernel::message::Message;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};


#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Debug, Clone)]
struct ActorCellInner {
  system: ActorSystem,
  my_self: Arc<dyn InternalActorRef>,
  path: ActorPath,
  mailbox: Arc<dyn AnyMessageSender>,
  system_mailbox: MailboxSender<SystemMessage>,
}

impl<M: Message> ActorContext<M> for ActorCellInner {
  fn my_self(&self) -> Arc<dyn ActorRef> {
    self.my_self.clone().to_actor_ref()
  }

  fn parent(&self) -> Arc<dyn ActorRef> {
    todo!()
  }
}

impl ActorCell {
  pub fn new(
    system: ActorSystem,
    my_self: Arc<dyn InternalActorRef>,
    path: ActorPath,
    mailbox: Arc<dyn AnyMessageSender>,
    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      inner: Arc::from(ActorCellInner {
        system,
        my_self,
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
