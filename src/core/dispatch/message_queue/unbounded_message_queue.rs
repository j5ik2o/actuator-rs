use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::message_queue::{
  MessageQueueBehavior, MessageQueueRWFactoryBehavior, MessageQueueReaderBehavior, MessageQueueReaderFactoryBehavior,
  MessageQueueSize, MessageQueueWithRWFactoryBehavior, MessageQueueWriterBehavior, MessageQueueWriterFactoryBehavior,
};
use crate::infrastructure::queue::{
  create_queue, Queue, QueueBehavior, QueueReader, QueueReaderBehavior, QueueReaderFactoryBehavior, QueueSize,
  QueueType, QueueWriter, QueueWriterBehavior, QueueWriterFactoryBehavior,
};

#[derive(Debug, Clone)]
pub struct UnboundedMessageQueue<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: Queue<Envelope>,
}

#[derive(Debug, Clone)]
pub struct UnboundedMessageQueueWriter<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: Queue<Envelope>,
  writer: QueueWriter<Envelope>,
}

#[derive(Debug, Clone)]
pub struct UnboundedMessageQueueReader<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: Queue<Envelope>,
  reader: QueueReader<Envelope>,
}

impl UnboundedMessageQueue<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> UnboundedMessageQueue<Msg> {
    UnboundedMessageQueue {
      _phantom: std::marker::PhantomData,
      queue: self.queue,
    }
  }
}

impl<Msg: Message> PartialEq for UnboundedMessageQueue<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.queue == other.queue
  }
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

  pub fn to_any(self) -> UnboundedMessageQueue<AnyMessage> {
    UnboundedMessageQueue {
      _phantom: std::marker::PhantomData,
      queue: self.queue,
    }
  }
}

impl<Msg: Message> MessageQueueBehavior<Msg> for UnboundedMessageQueue<Msg> {
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

impl<Msg: Message> MessageQueueRWFactoryBehavior<Msg> for UnboundedMessageQueue<Msg> {}

impl<Msg: Message> MessageQueueWriterFactoryBehavior<Msg> for UnboundedMessageQueue<Msg> {
  type Writer = UnboundedMessageQueueWriter<Msg>;

  fn writer(&self) -> Self::Writer {
    UnboundedMessageQueueWriter {
      _phantom: std::marker::PhantomData,
      queue: self.queue.clone(),
      writer: self.queue.writer(),
    }
  }
}

impl<Msg: Message> MessageQueueReaderFactoryBehavior<Msg> for UnboundedMessageQueue<Msg> {
  type Reader = UnboundedMessageQueueReader<Msg>;

  fn reader(&self) -> Self::Reader {
    UnboundedMessageQueueReader {
      _phantom: std::marker::PhantomData,
      queue: self.queue.clone(),
      reader: self.queue.reader(),
    }
  }
}

impl<Msg: Message> MessageQueueWithRWFactoryBehavior<Msg> for UnboundedMessageQueue<Msg> {}

// ---

impl<Msg: Message> MessageQueueBehavior<Msg> for UnboundedMessageQueueWriter<Msg> {
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

impl<Msg: Message> MessageQueueWriterBehavior<Msg> for UnboundedMessageQueueWriter<Msg> {
  fn enqueue(&mut self, _receiver: ActorRef<Msg>, handle: Envelope) -> anyhow::Result<()> {
    self.writer.offer(handle)
  }
}

// ---

impl<Msg: Message> MessageQueueBehavior<Msg> for UnboundedMessageQueueReader<Msg> {
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

impl<Msg: Message> MessageQueueReaderBehavior<Msg> for UnboundedMessageQueueReader<Msg> {
  fn dequeue(&mut self) -> anyhow::Result<Option<Envelope>> {
    self.reader.poll()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::core::actor::actor_ref::ActorRef;
  use std::sync::Arc;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn unbounded_message_queue_test() {
    let queue = UnboundedMessageQueue::<usize>::of_mpsc();
    let mut writer = queue.writer();
    let mut reader = queue.reader();

    let msg = 42;
    let actor_ref = ActorRef::NoSender;

    // メッセージをキューに追加
    writer.enqueue(actor_ref.clone(), Envelope::new(msg)).unwrap();

    // キューにメッセージが存在することを確認
    assert!(queue.has_messages());

    // キューからメッセージを取得
    let envelope = reader.dequeue().unwrap().unwrap();
    assert_eq!(envelope.typed_message::<usize>().unwrap(), msg);
    // assert_eq!(envelope.sender().unwrap(), actor_ref.to_any());

    // キューが空になったことを確認
    assert!(!queue.has_messages());
  }

  #[test]
  fn unbounded_message_queue_multithreaded_test() {
    let queue = Arc::new(UnboundedMessageQueue::<usize>::of_mpsc());
    let queue_clone = queue.clone();

    let writer_thread = thread::spawn(move || {
      let mut writer = queue_clone.writer();
      for i in 0..10 {
        let actor_ref = ActorRef::NoSender;
        writer.enqueue(actor_ref.clone(), Envelope::new(i)).unwrap();
        thread::sleep(Duration::from_millis(10));
      }
    });

    let reader_thread = thread::spawn(move || {
      let mut reader = queue.reader();
      for _ in 0..10 {
        while let Ok(None) = reader.dequeue() {
          log::debug!("waiting...");
          thread::sleep(Duration::from_millis(5));
        }
      }
    });

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
  }
}
