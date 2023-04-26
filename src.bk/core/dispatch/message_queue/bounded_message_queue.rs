use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::message_queue::{
  MessageQueueBehavior, MessageQueueRWFactoryBehavior, MessageQueueReaderBehavior, MessageQueueReaderFactoryBehavior,
  MessageQueueSize, MessageQueueWithRWFactoryBehavior, MessageQueueWriterBehavior, MessageQueueWriterFactoryBehavior,
};
use crate::infrastructure::queue::blocking_queue::{BlockingQueue, BlockingQueueReader, BlockingQueueWriter};
use crate::infrastructure::queue::{
  create_queue, BlockingQueueReaderBehavior, BlockingQueueReaderFactoryBehavior, BlockingQueueWriterBehavior,
  BlockingQueueWriterFactoryBehavior, Queue, QueueBehavior, QueueSize, QueueType,
};

#[derive(Debug, Clone)]
pub struct BoundedMessageQueue<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: BlockingQueue<Envelope, Queue<Envelope>>,
}

unsafe impl<Msg: Message> Send for BoundedMessageQueue<Msg> {}
unsafe impl<Msg: Message> Sync for BoundedMessageQueue<Msg> {}

impl<Msg: Message> PartialEq for BoundedMessageQueue<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.queue == other.queue
  }
}

impl BoundedMessageQueue<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> BoundedMessageQueue<Msg> {
    BoundedMessageQueue {
      _phantom: std::marker::PhantomData,
      queue: self.queue,
    }
  }
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

  pub fn to_any(self) -> BoundedMessageQueue<AnyMessage> {
    BoundedMessageQueue {
      _phantom: std::marker::PhantomData,
      queue: self.queue,
    }
  }
}

#[derive(Debug, Clone)]
pub struct BoundedMessageQueueWriter<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: BlockingQueue<Envelope, Queue<Envelope>>,
  writer: BlockingQueueWriter<Envelope, Queue<Envelope>>,
}

unsafe impl<Msg: Message> Send for BoundedMessageQueueWriter<Msg> {}
unsafe impl<Msg: Message> Sync for BoundedMessageQueueWriter<Msg> {}

#[derive(Debug, Clone)]
pub struct BoundedMessageQueueReader<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  queue: BlockingQueue<Envelope, Queue<Envelope>>,
  reader: BlockingQueueReader<Envelope, Queue<Envelope>>,
}

unsafe impl<Msg: Message> Send for BoundedMessageQueueReader<Msg> {}
unsafe impl<Msg: Message> Sync for BoundedMessageQueueReader<Msg> {}

impl<Msg: Message> MessageQueueBehavior<Msg> for BoundedMessageQueue<Msg> {
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

impl<Msg: Message> MessageQueueRWFactoryBehavior<Msg> for BoundedMessageQueue<Msg> {}

impl<Msg: Message> MessageQueueWriterFactoryBehavior<Msg> for BoundedMessageQueue<Msg> {
  type Writer = BoundedMessageQueueWriter<Msg>;

  fn writer(&self) -> Self::Writer {
    BoundedMessageQueueWriter {
      _phantom: std::marker::PhantomData,
      queue: self.queue.clone(),
      writer: self.queue.writer(),
    }
  }
}

impl<Msg: Message> MessageQueueReaderFactoryBehavior<Msg> for BoundedMessageQueue<Msg> {
  type Reader = BoundedMessageQueueReader<Msg>;

  fn reader(&self) -> Self::Reader {
    BoundedMessageQueueReader {
      _phantom: std::marker::PhantomData,
      queue: self.queue.clone(),
      reader: self.queue.reader(),
    }
  }
}

impl<Msg: Message> MessageQueueWithRWFactoryBehavior<Msg> for BoundedMessageQueue<Msg> {}

// ---

impl<Msg: Message> MessageQueueBehavior<Msg> for BoundedMessageQueueWriter<Msg> {
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

impl<Msg: Message> MessageQueueWriterBehavior<Msg> for BoundedMessageQueueWriter<Msg> {
  fn enqueue(&mut self, _receiver: ActorRef<Msg>, handle: Envelope) -> anyhow::Result<()> {
    self.writer.put(handle)
  }
}

// ---

impl<Msg: Message> MessageQueueBehavior<Msg> for BoundedMessageQueueReader<Msg> {
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

impl<Msg: Message> MessageQueueReaderBehavior<Msg> for BoundedMessageQueueReader<Msg> {
  fn dequeue(&mut self) -> anyhow::Result<Option<Envelope>> {
    self.reader.take()
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
    let queue = BoundedMessageQueue::<usize>::of_mpsc();
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
    let queue = Arc::new(BoundedMessageQueue::<usize>::of_vec_with_num_elements(2));
    let queue_clone = queue.clone();

    let writer_thread = thread::spawn(move || {
      log::debug!("writer_thread start");
      let mut writer = queue_clone.writer();
      log::debug!("writer_thread: writer()");
      for i in 0..10 {
        let actor_ref = ActorRef::NoSender;
        writer.enqueue(actor_ref.clone(), Envelope::new(i)).unwrap();
        log::debug!("writer_thread: i = {}", i);
        thread::sleep(Duration::from_millis(10));
      }
    });

    let reader_thread = thread::spawn(move || {
      log::debug!("reader_thread start");
      let mut reader = queue.reader();
      log::debug!("reader_thread: reader()");
      for _ in 0..10 {
        log::debug!(
          "reader_thread: dequeue(), size = {:?}: start",
          reader.number_of_messages()
        );
        let result = reader.dequeue();
        log::debug!("reader_thread: dequeue(): finished: result = {:?}", result);
        if let Ok(Some(envelope)) = result {
          let msg = envelope.typed_message::<i32>().unwrap();
          log::debug!("msg = {}", msg);
        }
      }
    });

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
  }
}
