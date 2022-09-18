use std::any::Any;

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::envelope::Envelope;

pub enum MessageQueue {
  Unbounded(UnboundedMessageQueue),
}

pub trait MessageQueueBehavior {
  fn enqueue(self, receiver: ActorRef, handle: Envelope);
  fn dequeue(self) -> Envelope;
}

impl MessageQueue {}

impl MessageQueueBehavior for MessageQueue {
  fn enqueue(self, receiver: ActorRef, handle: Envelope) {
    todo!()
  }

  fn dequeue(self) -> Envelope {
    todo!()
  }
}

pub struct UnboundedMessageQueue {}

impl MessageQueueBehavior for UnboundedMessageQueue {
  fn enqueue(self, receiver: ActorRef, handle: Envelope) {
    todo!()
  }

  fn dequeue(self) -> Envelope {
    todo!()
  }
}
