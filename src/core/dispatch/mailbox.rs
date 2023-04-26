use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::dispatcher::Dispatcher;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::message_queue::MessageQueueSize;
use anyhow::Result;

pub(crate) mod dead_letter;
pub(crate) mod dead_letter_mailbox;
pub(crate) mod mailbox;
pub(crate) mod mailbox_status;
pub(crate) mod mailbox_type;
pub(crate) mod system_mailbox;

pub trait MailboxBehavior<Msg: Message> {
  fn number_of_messages(&self) -> MessageQueueSize;
  fn has_messages(&self) -> bool;
  // async fn process_all_system_messages(&mut self);
  // async fn process_mailbox(&mut self);
}

#[async_trait::async_trait]
pub trait MailboxReaderBehavior<Msg: Message>: MailboxBehavior<Msg> {
  fn dequeue(&mut self) -> Result<Option<Envelope>>;
  async fn execute(&mut self, actor_cell: ActorCellWithRef<Msg>, dispatcher: Dispatcher);
}

pub trait MailboxWriterBehavior<Msg: Message>: MailboxBehavior<Msg> {
  fn enqueue(&mut self, receiver: ActorRef<Msg>, msg: Envelope) -> Result<()>;
}
