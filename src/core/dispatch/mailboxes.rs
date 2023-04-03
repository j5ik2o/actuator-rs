use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::mailbox::dead_letter_mailbox::DeadLetterMailbox;
use crate::core::dispatch::mailbox::mailbox::Mailbox;
use crate::core::dispatch::mailbox::mailbox_type::{MailboxType, MailboxTypeBehavior};

#[derive(Debug, Clone)]
pub struct Mailboxes {
  dead_letters: ActorRef<AnyMessage>,
  dead_letter_mailbox: DeadLetterMailbox,
}

impl Mailboxes {
  pub fn new(mailbox_type: MailboxType, dead_letters: ActorRef<AnyMessage>) -> Self {
    let mq = mailbox_type.create_message_queue(None);
    let mailbox = Mailbox::new_with_message_queue(mailbox_type, mq);
    let dead_letter_mailbox = DeadLetterMailbox::new(dead_letters.clone(), mailbox);
    Self {
      dead_letters,
      dead_letter_mailbox,
    }
  }

  pub fn dead_letter_mailbox(&self) -> DeadLetterMailbox {
    self.dead_letter_mailbox.clone()
  }
}
