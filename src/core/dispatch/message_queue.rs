use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::message::Message;

use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message_queue::bounded_message_queue::{
  BoundedMessageQueue, BoundedMessageQueueReader, BoundedMessageQueueWriter,
};
use crate::core::dispatch::message_queue::dead_letters_message_queue::{
  DeadLettersMessageQueue, DeadLettersMessageQueueReader, DeadLettersMessageQueueWriter,
};
use crate::core::dispatch::message_queue::unbounded_message_queue::{
  UnboundedMessageQueue, UnboundedMessageQueueReader, UnboundedMessageQueueWriter,
};
use crate::infrastructure::queue::{create_queue, Queue, QueueType};
use anyhow::Result;

mod bounded_message_queue;
mod dead_letters_message_queue;
mod unbounded_message_queue;

#[derive(Debug, Clone)]
pub enum MessageQueueSize {
  Limited(usize),
  Limitless,
}

pub trait MessageQueueBehavior<Msg: Message> {
  fn number_of_messages(&self) -> MessageQueueSize;
  fn has_messages(&self) -> bool;
}

pub trait MessageQueueWriterFactoryBehavior<Msg: Message> {
  type Writer: MessageQueueWriterBehavior<Msg>;
  fn writer(&self) -> Self::Writer;
}

pub trait MessageQueueReaderFactoryBehavior<Msg: Message> {
  type Reader: MessageQueueReaderBehavior<Msg>;
  fn reader(&self) -> Self::Reader;
}

pub trait MessageQueueRWFactoryBehavior<Msg: Message>:
  MessageQueueWriterFactoryBehavior<Msg> + MessageQueueReaderFactoryBehavior<Msg> {
}
pub trait MessageQueueWithRWFactoryBehavior<Msg: Message>:
  MessageQueueBehavior<Msg> + MessageQueueRWFactoryBehavior<Msg> {
}

pub trait MessageQueueWriterBehavior<Msg: Message>: MessageQueueBehavior<Msg> {
  fn enqueue(&mut self, receiver: ActorRef<Msg>, handle: Envelope) -> Result<()>;
}

pub trait MessageQueueReaderBehavior<Msg: Message>: MessageQueueBehavior<Msg> {
  fn dequeue(&mut self) -> Result<Option<Envelope>>;
}

#[derive(Debug, Clone)]
pub enum MessageQueue<Msg: Message> {
  Unbounded(UnboundedMessageQueue<Msg>),
  Bounded(BoundedMessageQueue<Msg>),
  DeadLetter(DeadLettersMessageQueue),
}

impl<Msg: Message> PartialEq for MessageQueue<Msg> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (MessageQueue::Unbounded(l), MessageQueue::Unbounded(r)) => l == r,
      (MessageQueue::Bounded(l), MessageQueue::Bounded(r)) => l == r,
      _ => false,
    }
  }
}

unsafe impl<Msg: Message> Send for MessageQueue<Msg> {}
unsafe impl<Msg: Message> Sync for MessageQueue<Msg> {}

#[derive(Debug, Clone)]
pub enum MessageQueueWriter<Msg: Message> {
  Unbounded(UnboundedMessageQueueWriter<Msg>),
  Bounded(BoundedMessageQueueWriter<Msg>),
  DeadLetter(DeadLettersMessageQueueWriter),
}

unsafe impl<Msg: Message> Send for MessageQueueWriter<Msg> {}
unsafe impl<Msg: Message> Sync for MessageQueueWriter<Msg> {}

#[derive(Debug, Clone)]
pub enum MessageQueueReader<Msg: Message> {
  Unbounded(UnboundedMessageQueueReader<Msg>),
  Bounded(BoundedMessageQueueReader<Msg>),
  DeadLetter(DeadLettersMessageQueueReader),
}

unsafe impl<Msg: Message> Send for MessageQueueReader<Msg> {}
unsafe impl<Msg: Message> Sync for MessageQueueReader<Msg> {}

impl MessageQueue<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> MessageQueue<Msg> {
    match self {
      MessageQueue::Unbounded(queue) => MessageQueue::Unbounded(queue.to_typed()),
      MessageQueue::Bounded(queue) => MessageQueue::Bounded(queue.to_typed()),
      MessageQueue::DeadLetter(queue) => MessageQueue::DeadLetter(queue),
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

  pub fn of_dead_letters(dead_letters: ActorRef<AnyMessage>) -> Self {
    MessageQueue::DeadLetter(DeadLettersMessageQueue::new(dead_letters))
  }

  pub fn to_any(self) -> MessageQueue<AnyMessage> {
    match self {
      MessageQueue::Unbounded(queue) => MessageQueue::Unbounded(queue.to_any()),
      MessageQueue::Bounded(queue) => MessageQueue::Bounded(queue.to_any()),
      MessageQueue::DeadLetter(queue) => MessageQueue::DeadLetter(queue.clone()),
    }
  }
}

impl<Msg: Message> MessageQueueBehavior<Msg> for MessageQueue<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    match self {
      MessageQueue::Unbounded(queue) => queue.number_of_messages(),
      MessageQueue::Bounded(queue) => queue.number_of_messages(),
      MessageQueue::DeadLetter(queue) => queue.number_of_messages(),
    }
  }

  fn has_messages(&self) -> bool {
    match self {
      MessageQueue::Unbounded(queue) => queue.has_messages(),
      MessageQueue::Bounded(queue) => queue.has_messages(),
      MessageQueue::DeadLetter(queue) => queue.has_messages(),
    }
  }
}

impl<Msg: Message> MessageQueueRWFactoryBehavior<Msg> for MessageQueue<Msg> {}

impl<Msg: Message> MessageQueueWriterFactoryBehavior<Msg> for MessageQueue<Msg> {
  type Writer = MessageQueueWriter<Msg>;

  fn writer(&self) -> Self::Writer {
    match self {
      MessageQueue::Unbounded(queue) => MessageQueueWriter::Unbounded(queue.writer()),
      MessageQueue::Bounded(queue) => MessageQueueWriter::Bounded(queue.writer()),
      MessageQueue::DeadLetter(queue) => MessageQueueWriter::DeadLetter(queue.writer()),
    }
  }
}

impl<Msg: Message> MessageQueueReaderFactoryBehavior<Msg> for MessageQueue<Msg> {
  type Reader = MessageQueueReader<Msg>;

  fn reader(&self) -> Self::Reader {
    match self {
      MessageQueue::Unbounded(queue) => MessageQueueReader::Unbounded(queue.reader()),
      MessageQueue::Bounded(queue) => MessageQueueReader::Bounded(queue.reader()),
      MessageQueue::DeadLetter(queue) => MessageQueueReader::DeadLetter(queue.reader()),
    }
  }
}

impl<Msg: Message> MessageQueueWithRWFactoryBehavior<Msg> for MessageQueue<Msg> {}

// ---

impl<Msg: Message> MessageQueueBehavior<Msg> for MessageQueueWriter<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    match self {
      MessageQueueWriter::Unbounded(queue) => queue.number_of_messages(),
      MessageQueueWriter::Bounded(queue) => queue.number_of_messages(),
      MessageQueueWriter::DeadLetter(queue) => queue.number_of_messages(),
    }
  }

  fn has_messages(&self) -> bool {
    match self {
      MessageQueueWriter::Unbounded(queue) => queue.has_messages(),
      MessageQueueWriter::Bounded(queue) => queue.has_messages(),
      MessageQueueWriter::DeadLetter(queue) => queue.has_messages(),
    }
  }
}

impl<Msg: Message> MessageQueueWriterBehavior<Msg> for MessageQueueWriter<Msg> {
  fn enqueue(&mut self, receiver: ActorRef<Msg>, handle: Envelope) -> Result<()> {
    match self {
      MessageQueueWriter::Unbounded(queue) => queue.enqueue(receiver, handle),
      MessageQueueWriter::Bounded(queue) => queue.enqueue(receiver, handle),
      MessageQueueWriter::DeadLetter(queue) => queue.enqueue(receiver.to_any(), handle),
    }
  }
}

impl<Msg: Message> MessageQueueBehavior<Msg> for MessageQueueReader<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    match self {
      MessageQueueReader::Unbounded(queue) => queue.number_of_messages(),
      MessageQueueReader::Bounded(queue) => queue.number_of_messages(),
      MessageQueueReader::DeadLetter(queue) => queue.number_of_messages(),
    }
  }

  fn has_messages(&self) -> bool {
    match self {
      MessageQueueReader::Unbounded(queue) => queue.has_messages(),
      MessageQueueReader::Bounded(queue) => queue.has_messages(),
      MessageQueueReader::DeadLetter(queue) => queue.has_messages(),
    }
  }
}

impl<Msg: Message> MessageQueueReaderBehavior<Msg> for MessageQueueReader<Msg> {
  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    match self {
      MessageQueueReader::Unbounded(queue) => queue.dequeue(),
      MessageQueueReader::Bounded(queue) => queue.dequeue(),
      MessageQueueReader::DeadLetter(queue) => queue.dequeue(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::actor::actor_ref::ActorRefBehavior;

  #[test]
  fn test_message_queue_of_unbounded_with_queue_type() {
    let message_queue = MessageQueue::of_unbounded_with_queue_type(QueueType::Vec);

    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    let no_sender: ActorRef<String> = ActorRef::NoSender;
    let mut writer = message_queue.writer();
    writer.enqueue(no_sender.clone(), send_envelope.clone()).unwrap();

    let mut reader = message_queue.reader();
    let receive_envelope = reader.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>().unwrap();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn of_unbounded_with_queue_type_with_num_elements() {
    let message_queue = MessageQueue::of_unbounded_with_queue_type_with_num_elements(QueueType::Vec, 3);
    let no_sender: ActorRef<String> = ActorRef::NoSender;
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new_with_sender(send_message_text.clone(), no_sender.clone().to_any());

    let mut writer = message_queue.writer();
    writer.enqueue(no_sender, send_envelope.clone()).unwrap();

    let mut reader = message_queue.reader();
    let receive_envelope = reader.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>().unwrap();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn test_message_queue_of_bounded_with_queue_type() {
    let message_queue = MessageQueue::<String>::of_bounded_with_queue_type(QueueType::Vec);
    let no_sender: ActorRef<String> = ActorRef::NoSender;
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    let mut writer = message_queue.writer();
    writer.enqueue(no_sender, send_envelope.clone()).unwrap();

    let mut reader = message_queue.reader();
    let receive_envelope = reader.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>().unwrap();

    assert_eq!(receive_text, send_message_text);
  }

  #[test]
  fn test_message_queue_of_bounded_with_queue_type_with_num_elements() {
    let message_queue = MessageQueue::<String>::of_bounded_with_queue_type_with_num_elements(QueueType::Vec, 3);
    let no_sender: ActorRef<String> = ActorRef::NoSender;
    let send_message_text = "message".to_owned();
    let send_envelope = Envelope::new(send_message_text.clone());

    let mut writer = message_queue.writer();
    writer.enqueue(no_sender, send_envelope.clone()).unwrap();

    let mut reader = message_queue.reader();
    let receive_envelope = reader.dequeue().unwrap().unwrap();

    let receive_text = receive_envelope.typed_message::<String>().unwrap();

    assert_eq!(receive_text, send_message_text);
  }
}
