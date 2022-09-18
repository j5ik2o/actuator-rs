use crate::actor::actor_ref::ActorRef;
use crate::dispatch::envelope::Envelope;
use crate::queue::{
  create_queue, BlockingQueue, BlockingQueueBehavior, Element, Queue, QueueBehavior, QueueSize, QueueType,
};
use anyhow::Result;
use regex::internal::Input;

pub enum MessageQueue {
  Unbounded(UnboundedMessageQueue),
  Bounded(BoundedMessageQueue),
}

pub enum MessageQueueSize {
  Limited(usize),
  Limitless,
}

pub trait MessageQueueBehavior {
  type ElementType: Element;
  type QueueType: QueueBehavior<Self::ElementType>;

  fn queue(&self) -> &Self::QueueType;

  fn queue_mut(&mut self) -> &mut Self::QueueType;

  fn enqueue(&mut self, _: ActorRef, handle: Self::ElementType) -> Result<()> {
    self.queue_mut().offer(handle)
  }

  fn dequeue(&mut self) -> Result<Option<Self::ElementType>> {
    self.queue_mut().poll()
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    match self.queue().len() {
      QueueSize::Limited(n) => MessageQueueSize::Limited(n),
      QueueSize::Limitless => MessageQueueSize::Limitless,
    }
  }

  fn has_messages(&self) -> bool {
    self.queue().non_empty()
  }

  fn clean_up(&mut self, owner: ActorRef, dead_letters: &mut Self) {
    if self.has_messages() {
      while let Ok(Some(envelope)) = self.dequeue() {
        dead_letters.enqueue(owner.clone(), envelope).unwrap();
      }
    }
  }
}

impl MessageQueue {
  pub fn of_unbounded(queue: Queue<Envelope>) -> Self {
    MessageQueue::Unbounded(UnboundedMessageQueue::new(queue))
  }

  pub fn of_unbounded_with_queue_type(queue_type: QueueType) -> Self {
    MessageQueue::Unbounded(UnboundedMessageQueue::new(create_queue(queue_type, None)))
  }

  pub fn of_unbounded_with_queue_type_with_num_elements(queue_type: QueueType, num_elements: usize) -> Self {
    MessageQueue::Unbounded(UnboundedMessageQueue::new(create_queue(queue_type, Some(num_elements))))
  }

  pub fn of_bounded(queue: Queue<Envelope>) -> Self {
    MessageQueue::Bounded(BoundedMessageQueue::new_with_queue(queue))
  }

  pub fn of_bounded_with_queue_type(queue_type: QueueType) -> Self {
    MessageQueue::Bounded(BoundedMessageQueue::new_with_queue(create_queue(queue_type, None)))
  }

  pub fn of_bounded_with_queue_type_with_num_elements(queue_type: QueueType, num_elements: usize) -> Self {
    MessageQueue::Bounded(BoundedMessageQueue::new_with_queue(create_queue(
      queue_type,
      Some(num_elements),
    )))
  }
}

impl MessageQueueBehavior for MessageQueue {
  type ElementType = Envelope;
  type QueueType = Queue<Self::ElementType>;

  fn queue(&self) -> &Self::QueueType {
    match self {
      MessageQueue::Unbounded(inner) => &inner.queue,
      _ => panic!(""),
    }
  }

  fn queue_mut(&mut self) -> &mut Self::QueueType {
    match self {
      MessageQueue::Unbounded(inner) => &mut inner.queue,
      _ => panic!(""),
    }
  }
}

pub struct UnboundedMessageQueue {
  queue: Queue<Envelope>,
}

impl UnboundedMessageQueue {
  pub fn of_mpsc() -> Self {
    Self::new(create_queue(QueueType::MPSC, None))
  }

  pub fn of_mpsc_with_num_elements(num_elements: usize) -> Self {
    Self::new(create_queue(QueueType::MPSC, Some(num_elements)))
  }

  pub fn of_vec() -> Self {
    Self::new(create_queue(QueueType::Vec, None))
  }

  pub fn of_vec_with_num_elements(num_elements: usize) -> Self {
    Self::new(create_queue(QueueType::Vec, Some(num_elements)))
  }

  pub fn new(queue: Queue<Envelope>) -> Self {
    Self { queue }
  }
}

impl MessageQueueBehavior for UnboundedMessageQueue {
  type ElementType = Envelope;
  type QueueType = Queue<Self::ElementType>;

  fn queue(&self) -> &Queue<Envelope> {
    &self.queue
  }

  fn queue_mut(&mut self) -> &mut Queue<Envelope> {
    &mut self.queue
  }
}

pub struct BoundedMessageQueue {
  queue: BlockingQueue<Envelope, Queue<Envelope>>,
}

impl BoundedMessageQueue {
  pub fn of_mpsc() -> Self {
    Self::new(create_queue(QueueType::MPSC, None).with_blocking())
  }

  pub fn of_mpsc_with_num_elements(num_elements: usize) -> Self {
    Self::new(create_queue(QueueType::MPSC, Some(num_elements)).with_blocking())
  }

  pub fn of_vec() -> Self {
    Self::new(create_queue(QueueType::Vec, None).with_blocking())
  }

  pub fn of_vec_with_num_elements(num_elements: usize) -> Self {
    Self::new(create_queue(QueueType::Vec, Some(num_elements)).with_blocking())
  }

  pub fn new(queue: BlockingQueue<Envelope, Queue<Envelope>>) -> Self {
    Self { queue }
  }

  pub fn new_with_queue(queue: Queue<Envelope>) -> Self {
    Self::new(queue.with_blocking())
  }
}

impl MessageQueueBehavior for BoundedMessageQueue {
  type ElementType = Envelope;
  type QueueType = BlockingQueue<Self::ElementType, Queue<Self::ElementType>>;

  fn queue(&self) -> &Self::QueueType {
    &self.queue
  }

  fn queue_mut(&mut self) -> &mut Self::QueueType {
    &mut self.queue
  }

  fn enqueue(&mut self, _: ActorRef, handle: Self::ElementType) -> Result<()> {
    self.queue_mut().put(handle)
  }

  fn dequeue(&mut self) -> Result<Option<Self::ElementType>> {
    self.queue_mut().take()
  }
}

#[cfg(test)]
mod tests {
  use crate::actor::actor_ref::ActorRef;
  use crate::dispatch::any_message::AnyMessage;
  use crate::dispatch::envelope::Envelope;
  use crate::dispatch::message_queue::{MessageQueue, MessageQueueBehavior};
  use crate::queue::QueueType;
  use std::sync::Arc;

  #[test]
  fn test_1() {
    let mut mq = MessageQueue::of_unbounded_with_queue_type(QueueType::Vec);
    let str = "message".to_owned();
    let msg = AnyMessage::new(str.clone(), false);
    let mut envelope = Envelope::new(Some(msg), ActorRef::NoSender.clone());
    let _ = mq.enqueue(ActorRef::NoSender.clone(), envelope.clone()).unwrap();
    let _r = mq.dequeue().unwrap();
    let m = envelope.message_mut();
    let result = m.take::<String>().unwrap();
    assert_eq!(result, Arc::new(str));
    println!("{:?}", result);
  }
}
