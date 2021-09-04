use std::fmt::Debug;
use std::sync::Arc;

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_ref::ActorRef;
use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox::Mailbox;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::message::Message;
use crate::kernel::message_dispatcher::MessageDispatcher;

#[derive(Debug)]
pub struct AnyEnqueueError;

impl From<()> for AnyEnqueueError {
  fn from(_: ()) -> AnyEnqueueError {
    AnyEnqueueError
  }
}

pub trait AnyMessageSender: Debug + Send + Sync {
  fn try_enqueue_any(&self, receiver: Arc<dyn ActorRef>, msg: AnyMessage, sender: Sender) -> Result<(), AnyEnqueueError>;

  fn actor(&self) -> ActorCell;
  fn dispatcher(&self) -> Arc<dyn MessageDispatcher>;

  fn set_as_scheduled(&mut self) -> bool;
  fn set_as_idle(&mut self) -> bool;
  fn is_scheduled(&self) -> bool;
  fn is_closed(&self) -> bool;

  fn suspend(&self) -> bool;
  fn resume(&self) -> bool;
}

impl<M: Message> AnyMessageSender for MailboxSender<M> {
  fn try_enqueue_any(
    &self,
    receiver: Arc<dyn ActorRef>,
    any_message: AnyMessage,
    sender: Sender,
  ) -> Result<(), AnyEnqueueError> {
    let mut msg = any_message;
    let actual = msg.take().map_err(|_| AnyEnqueueError)?;
    let msg = Envelope {
      message: actual,
      sender,
    };
    self.try_enqueue(receiver, msg).map_err(|_| AnyEnqueueError)
  }

  fn actor(&self) -> ActorCell {
    self.to_mailbox().get_actor().unwrap().actor_cell().clone()
  }

  fn dispatcher(&self) -> Arc<dyn MessageDispatcher> {
    todo!()
  }

  fn set_as_scheduled(&mut self) -> bool {
    self.set_as_scheduled()
  }

  fn set_as_idle(&mut self) -> bool {
    self.set_as_idle()
  }

  fn is_scheduled(&self) -> bool {
    self.is_scheduled()
  }

  fn is_closed(&self) -> bool {
    self.is_closed()
  }

  fn suspend(&self) -> bool {
    self.suspend()
  }

  fn resume(&self) -> bool {
    self.resume()
  }
}
