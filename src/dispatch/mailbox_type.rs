use crate::actor::actor_ref::ActorRef;
use crate::dispatch::message_queue::MessageQueue;
use crate::queue::QueueType;
use std::time::Duration;

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

pub trait MailboxTypeBehavior {
  fn create(&self, owner: Option<ActorRef> /* system: Option<ActorSystem> */) -> MessageQueue;
}

impl MailboxTypeBehavior for MailboxType {
  fn create(&self, _owner: Option<ActorRef>) -> MessageQueue {
    match self {
      MailboxType::Unbounded => MessageQueue::of_unbounded_with_queue_type(QueueType::MPSC),
      MailboxType::Bounded { capacity, .. } => {
        MessageQueue::of_bounded_with_queue_type_with_num_elements(QueueType::MPSC, *capacity)
      }
      _ => panic!(""),
    }
  }
}
