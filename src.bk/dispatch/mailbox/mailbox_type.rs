use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::dispatch::message::Message;
use crate::dispatch::message_queue::MessageQueue;

use std::time::Duration;

use crate::infrastructure::queue::QueueType;

#[derive(Debug, Clone, PartialEq)]
pub enum MailboxType {
  Unbounded,
  Bounded { capacity: usize, push_time_out: Duration },
}

impl MailboxType {
  pub fn of_unbounded() -> Self {
    MailboxType::Unbounded
  }

  pub fn of_bounded(capacity: usize, push_time_out: Duration) -> Self {
    MailboxType::Bounded {
      capacity,
      push_time_out,
    }
  }
}

pub trait MailboxTypeBehavior<Msg: Message> {
  fn create_message_queue(
    &self,
    owner: Option<ActorRefRef<Msg>>, // system: Option<ActorSystem>
  ) -> MessageQueue<Msg>;
}

impl<Msg: Message> MailboxTypeBehavior<Msg> for MailboxType {
  fn create_message_queue(&self, _owner: Option<ActorRefRef<Msg>>) -> MessageQueue<Msg> {
    match self {
      MailboxType::Unbounded => MessageQueue::of_unbounded_with_queue_type(QueueType::MPSC),
      MailboxType::Bounded { capacity, .. } => {
        MessageQueue::of_bounded_with_queue_type_with_num_elements(QueueType::MPSC, *capacity)
      }
    }
  }
}
