use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::message::Message;
use crate::infrastructure::queue::{
  create_queue, BlockingQueue, BlockingQueueBehavior, Queue, QueueBehavior, QueueSize, QueueType,
};
use anyhow::Result;
use std::fmt::Debug;

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;

pub trait MessageQueueBehavior: Debug {
  type ElementType: Message;

  fn enqueue(&mut self, receiver: ActorRefRef<Self::ElementType>, handle: Envelope) -> Result<()>;

  fn dequeue(&mut self) -> Result<Option<Envelope>>;

  fn number_of_messages(&self) -> MessageQueueSize;

  fn has_messages(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum MessageQueue<Msg: Message> {
  Unbounded(UnboundedMessageQueue<Msg>),
  Bounded(BoundedMessageQueue<Msg>),
  DeadLetters(DeadLettersMessageQueue<Msg>),
}

pub enum MessageQueueSize {
  Limited(usize),
  Limitless,
}

pub trait CleanUp: MessageQueueBehavior {
  fn clean_up(&mut self, owner: ActorRefRef<Self::ElementType>, dead_letters: &mut Self) {
    if self.has_messages() {
      while let Ok(Some(envelope)) = self.dequeue() {
        dead_letters.enqueue(owner.clone(), envelope).unwrap();
      }
    }
  }
}

impl<Msg: Message> MessageQueue<Msg> {
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

  pub fn of_dead_letters(dead_letters: ActorRefRef<Msg>) -> Self {
    MessageQueue::DeadLetters(DeadLettersMessageQueue::new(dead_letters))
  }
}

impl<Msg: Message> MessageQueueBehavior for MessageQueue<Msg> {
  type ElementType = Msg;

  fn enqueue(&mut self, receiver: ActorRefRef<Self::ElementType>, handle: Envelope) -> Result<()> {
    match self {
      MessageQueue::Unbounded(inner) => inner.enqueue(receiver, handle),
      MessageQueue::Bounded(inner) => inner.enqueue(receiver, handle),
      MessageQueue::DeadLetters(inner) => inner.enqueue(receiver, handle),
    }
  }

  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    match self {
      MessageQueue::Unbounded(inner) => inner.dequeue(),
      MessageQueue::Bounded(inner) => inner.dequeue(),
      MessageQueue::DeadLetters(inner) => inner.dequeue(),
    }
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    match self {
      MessageQueue::Unbounded(inner) => inner.number_of_messages(),
      MessageQueue::Bounded(inner) => inner.number_of_messages(),
      MessageQueue::DeadLetters(inner) => inner.number_of_messages(),
    }
  }

  fn has_messages(&self) -> bool {
    match self {
      MessageQueue::Unbounded(inner) => inner.has_messages(),
      MessageQueue::Bounded(inner) => inner.has_messages(),
      MessageQueue::DeadLetters(inner) => inner.has_messages(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct UnboundedMessageQueue<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: Queue<Envelope>,
}

impl<Msg: Message> UnboundedMessageQueue<Msg> {
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
    Self {
      _phantom: std::marker::PhantomData,
      queue,
    }
  }
}

impl<Msg: Message> MessageQueueBehavior for UnboundedMessageQueue<Msg> {
  type ElementType = Msg;

  fn enqueue(&mut self, _receiver: ActorRefRef<Self::ElementType>, handle: Envelope) -> Result<()> {
    self.queue.offer(handle)
  }

  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    self.queue.poll()
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    match self.queue.len() {
      QueueSize::Limited(n) => MessageQueueSize::Limited(n),
      QueueSize::Limitless => MessageQueueSize::Limitless,
    }
  }

  fn has_messages(&self) -> bool {
    self.queue.non_empty()
  }
}

#[derive(Debug, Clone)]
pub struct BoundedMessageQueue<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: BlockingQueue<Envelope, Queue<Envelope>>,
}

impl<Msg: Message> BoundedMessageQueue<Msg> {
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
    Self {
      _phantom: std::marker::PhantomData,
      queue,
    }
  }

  pub fn new_with_queue(queue: Queue<Envelope>) -> Self {
    Self::new(queue.with_blocking())
  }
}

impl<Msg: Message> MessageQueueBehavior for BoundedMessageQueue<Msg> {
  type ElementType = Msg;

  fn enqueue(&mut self, _receiver: ActorRefRef<Self::ElementType>, handle: Envelope) -> Result<()> {
    self.queue.put(handle)
  }

  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    self.queue.take()
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    match self.queue.len() {
      QueueSize::Limited(n) => MessageQueueSize::Limited(n),
      QueueSize::Limitless => MessageQueueSize::Limitless,
    }
  }

  fn has_messages(&self) -> bool {
    self.queue.non_empty()
  }
}

#[derive(Debug, Clone)]
pub struct DeadLettersMessageQueue<Msg: Message> {
  dead_letters: ActorRefRef<Msg>,
}

impl<Msg: Message> DeadLettersMessageQueue<Msg> {
  pub fn new(dead_letters: ActorRefRef<Msg>) -> Self {
    Self { dead_letters }
  }
}

#[derive(Debug, Clone)]
pub struct DeadLetter<Msg: Message> {
  message: AnyMessage,
  sender: ActorRefRef<Msg>,
  recipient: ActorRefRef<Msg>,
}

unsafe impl<Msg: Message> Send for DeadLetter<Msg> {}
unsafe impl<Msg: Message> Sync for DeadLetter<Msg> {}

impl<Msg: Message> DeadLetter<Msg> {
  pub fn new(message: AnyMessage, sender: ActorRefRef<Msg>, recipient: ActorRefRef<Msg>) -> Self {
    Self {
      message,
      sender,
      recipient,
    }
  }
}

impl<Msg: Message> PartialEq for DeadLetter<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.message == other.message && self.sender == other.sender && self.recipient == other.recipient
  }
}

impl<Msg: Message> MessageQueueBehavior for DeadLettersMessageQueue<Msg> {
  type ElementType = Msg;

  fn enqueue(&mut self, receiver: ActorRefRef<Self::ElementType>, handle: Envelope) -> Result<()> {
    let dead_letters = self.dead_letters.to_untyped_actor_ref();
    let mut dead_letters_guard = dead_letters.borrow_mut();
    let dead_letter = DeadLetter::new(AnyMessage::new(handle), receiver.clone(), receiver.clone());
    let any_message = AnyMessage::new(dead_letter);
    dead_letters_guard.tell_any(any_message);
    Ok(())
  }

  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    unimplemented!()
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    unimplemented!()
  }

  fn has_messages(&self) -> bool {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {

  use crate::actor::actor_ref::actor_ref::ActorRef;
  use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;

  use crate::dispatch::envelope::Envelope;
  use crate::dispatch::message_queue::{MessageQueue, MessageQueueBehavior};
  use crate::infrastructure::queue::QueueType;

  #[test]
  fn test_message_queue_of_unbounded_with_queue_type() {
    let mut message_queue = MessageQueue::of_unbounded_with_queue_type(QueueType::Vec);

    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    let no_sender: ActorRefRef<String> = ActorRefRef::new(ActorRef::<String>::NoSender);
    message_queue.enqueue(no_sender.clone(), send_envelope.clone()).unwrap();

    let receive_envelope = message_queue.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn of_unbounded_with_queue_type_with_num_elements() {
    let mut message_queue = MessageQueue::of_unbounded_with_queue_type_with_num_elements(QueueType::Vec, 3);
    let no_sender = ActorRefRef::new(ActorRef::<String>::NoSender);
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new_with_sender(send_message_text.clone(), no_sender.to_any_ref_ref());

    message_queue.enqueue(no_sender, send_envelope.clone()).unwrap();

    let receive_envelope = message_queue.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn test_message_queue_of_bounded_with_queue_type() {
    let mut message_queue = MessageQueue::<String>::of_bounded_with_queue_type(QueueType::Vec);
    let no_sender = ActorRefRef::new(ActorRef::NoSender);
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    message_queue.enqueue(no_sender, send_envelope.clone()).unwrap();

    let receive_envelope = message_queue.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn test_message_queue_of_bounded_with_queue_type_with_num_elements() {
    let mut message_queue = MessageQueue::<String>::of_bounded_with_queue_type_with_num_elements(QueueType::Vec, 3);
    let no_sender = ActorRefRef::new(ActorRef::NoSender);
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    message_queue.enqueue(no_sender, send_envelope.clone()).unwrap();

    let receive_envelope = message_queue.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>();

    assert_eq!(receive_text, send_message_text);
  }
}
