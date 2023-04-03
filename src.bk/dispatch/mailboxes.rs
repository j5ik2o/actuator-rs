use std::sync::{Arc, Mutex};

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::dead_letter_mailbox::DeadLetterMailbox;

#[derive(Debug, Clone)]
pub struct Mailboxes {
  dead_letters: ActorRefRef<AnyMessage>,
  dead_letter_mailbox: Arc<Mutex<DeadLetterMailbox>>,
}

impl Mailboxes {
  pub fn new(dead_letters: ActorRefRef<AnyMessage>) -> Self {
    let dead_letter_mailbox = Arc::new(Mutex::new(DeadLetterMailbox::new(dead_letters.clone())));
    Self {
      dead_letters,
      dead_letter_mailbox,
    }
  }

  pub fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>> {
    self.dead_letter_mailbox.clone()
  }
}
