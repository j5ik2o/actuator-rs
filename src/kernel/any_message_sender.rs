use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::message::Message;
use std::fmt::Debug;

#[derive(Debug)]
pub struct AnyEnqueueError;

impl From<()> for AnyEnqueueError {
  fn from(_: ()) -> AnyEnqueueError {
    AnyEnqueueError
  }
}

pub trait AnyMessageSender: Debug + Send + Sync {
  fn try_any_enqueue(&self, msg: &mut AnyMessage, sender: Sender) -> Result<(), AnyEnqueueError>;

  fn set_as_scheduled(&mut self) -> bool;
  fn set_as_idle(&mut self) -> bool;
  fn is_scheduled(&self) -> bool;
  fn is_closed(&self) -> bool;

  fn suspend(&self) -> bool;
  fn resume(&self) -> bool;
}

impl<M: Message> AnyMessageSender for MailboxSender<M> {
  fn try_any_enqueue(&self, msg: &mut AnyMessage, sender: Sender) -> Result<(), AnyEnqueueError> {
    let actual = msg.take().map_err(|_| AnyEnqueueError)?;
    let msg = Envelope {
      message: actual,
      sender,
    };
    self.try_enqueue(msg).map_err(|_| AnyEnqueueError)
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
