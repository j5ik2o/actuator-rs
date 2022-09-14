use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;

use std::sync::{Arc, Mutex};

use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox::Mailbox;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::mailbox::{MailboxBehavior, MailboxInternal};

use crate::dispatch::message_queue::{DeadLetter, MessageQueue, MessageQueueSize};
use crate::dispatch::system_message::{
  EarliestFirstSystemMessageList, LatestFirstSystemMessageList, SystemMessage, SystemMessageQueueBehavior, ENIL,
};

#[derive(Debug, Clone)]
pub struct DeadLetterMailbox {
  dead_letters: ActorRefRef<AnyMessage>,
  underlying: Mailbox<AnyMessage>,
}

impl DeadLetterMailbox {
  pub fn new(dead_letters: ActorRefRef<AnyMessage>) -> Self {
    let dlmq = Arc::new(Mutex::new(MessageQueue::of_dead_letters(dead_letters.clone())));
    let mut mailbox = Mailbox::new_with_message_queue(MailboxType::Unbounded, dlmq);
    mailbox.become_closed();
    Self {
      dead_letters,
      underlying: mailbox,
    }
  }
}

#[async_trait::async_trait]
impl MailboxBehavior<AnyMessage> for DeadLetterMailbox {
  fn number_of_messages(&self) -> MessageQueueSize {
    self.underlying.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    self.underlying.has_messages()
  }

  fn enqueue(&mut self, receiver: ActorRefRef<AnyMessage>, msg: Envelope) -> anyhow::Result<()> {
    self.underlying.enqueue(receiver, msg)
  }

  fn dequeue(&mut self) -> anyhow::Result<Envelope> {
    self.underlying.dequeue()
  }

  async fn process_all_system_messages(&mut self) {
    self.underlying.process_all_system_messages().await
  }

  async fn process_mailbox(&mut self) {
    self.underlying.process_mailbox().await
  }

  async fn execute(&mut self) {
    self.underlying.execute().await
  }
}

impl SystemMessageQueueBehavior<AnyMessage> for DeadLetterMailbox {
  fn system_enqueue(&mut self, receiver: ActorRefRef<AnyMessage>, message: &mut SystemMessage) {
    let dead_letters_arc_mutex = self.dead_letters.to_untyped_actor_ref();
    let mut dead_letters_guard = dead_letters_arc_mutex.borrow_mut();
    let dead_letter = DeadLetter::new(AnyMessage::new(message.clone()), receiver.clone(), receiver);
    let any_message = AnyMessage::new(dead_letter);
    dead_letters_guard.tell_any(any_message)
  }

  fn system_drain(&mut self, _new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    ENIL.clone()
  }

  fn has_system_messages(&self) -> bool {
    false
  }
}
