use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::mailbox::dead_letter::DeadLetter;
use crate::core::dispatch::mailbox::mailbox::Mailbox;
use crate::core::dispatch::mailbox::MailboxBehavior;
use crate::core::dispatch::message_queue::MessageQueueSize;
use crate::core::dispatch::system_message::earliest_first_system_message_list::EarliestFirstSystemMessageList;
use crate::core::dispatch::system_message::latest_first_system_message_list::LatestFirstSystemMessageList;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::{SystemMessageQueueReaderBehavior, SystemMessageQueueWriterBehavior, ENIL};

#[derive(Debug, Clone)]
pub struct DeadLetterMailbox {
  dead_letters: ActorRef<AnyMessage>,
  underlying: Mailbox<AnyMessage>,
}

impl DeadLetterMailbox {
  pub fn new(dead_letters: ActorRef<AnyMessage>, underlying: Mailbox<AnyMessage>) -> Self {
    Self {
      dead_letters,
      underlying,
    }
  }
}

impl MailboxBehavior<AnyMessage> for DeadLetterMailbox {
  fn number_of_messages(&self) -> MessageQueueSize {
    self.underlying.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    self.underlying.has_messages()
  }
}

impl SystemMessageQueueWriterBehavior<AnyMessage> for DeadLetterMailbox {
  fn system_enqueue(&mut self, receiver: ActorRef<AnyMessage>, message: &mut SystemMessageEntry) {
    let dead_letter = DeadLetter::new(AnyMessage::new(message.clone()), receiver.clone(), receiver);
    let any_message = AnyMessage::new(dead_letter);
    self.dead_letters.tell(any_message)
  }
}

impl SystemMessageQueueReaderBehavior for DeadLetterMailbox {
  fn has_system_messages(&self) -> bool {
    false
  }

  fn system_drain(&mut self, _new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    ENIL.clone()
  }
}
