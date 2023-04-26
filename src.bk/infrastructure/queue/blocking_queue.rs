use crate::infrastructure::queue::{
  BlockingQueueBehavior, BlockingQueueRWFactoryBehavior, BlockingQueueReaderBehavior,
  BlockingQueueReaderFactoryBehavior, BlockingQueueWithRWFactoryBehavior, BlockingQueueWriterBehavior,
  BlockingQueueWriterFactoryBehavior, Element, QueueBehavior, QueueReaderBehavior, QueueSize,
  QueueWithRWFactoryBehavior, QueueWriterBehavior,
};

use std::marker::PhantomData;

use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug, Clone)]
pub struct BlockingQueue<E: Element, Q: QueueWithRWFactoryBehavior<E>> {
  underlying: Arc<(Mutex<Q>, Condvar, Condvar)>,
  p: PhantomData<E>,
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> PartialEq for BlockingQueue<E, Q> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.underlying, &other.underlying)
  }
}

#[derive(Debug, Clone)]
pub struct BlockingQueueWriter<E: Element, Q: QueueWithRWFactoryBehavior<E>> {
  queue: BlockingQueue<E, Q>,
  writer: Q::Writer,
}

#[derive(Debug, Clone)]
pub struct BlockingQueueReader<E: Element, Q: QueueWithRWFactoryBehavior<E>> {
  queue: BlockingQueue<E, Q>,
  reader: Q::Reader,
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueue<E, Q> {
  pub fn new(queue: Q) -> Self {
    Self {
      underlying: Arc::new((Mutex::new(queue), Condvar::new(), Condvar::new())),
      p: PhantomData::default(),
    }
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueBehavior<E> for BlockingQueue<E, Q> {}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> QueueBehavior<E> for BlockingQueue<E, Q> {
  fn len(&self) -> QueueSize {
    let q = self.underlying.0.lock().unwrap();
    q.len()
  }

  fn capacity(&self) -> QueueSize {
    let q = self.underlying.0.lock().unwrap();
    q.capacity()
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueRWFactoryBehavior<E> for BlockingQueue<E, Q> {}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueWriterFactoryBehavior<E> for BlockingQueue<E, Q> {
  type Writer = BlockingQueueWriter<E, Q>;

  fn writer(&self) -> Self::Writer {
    let writer = self.underlying.0.lock().unwrap().writer();
    BlockingQueueWriter {
      queue: self.clone(),
      writer,
    }
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueReaderFactoryBehavior<E> for BlockingQueue<E, Q> {
  type Reader = BlockingQueueReader<E, Q>;

  fn reader(&self) -> Self::Reader {
    let reader = self.underlying.0.lock().unwrap().reader();
    BlockingQueueReader {
      queue: self.clone(),
      // reader: Arc::new(Mutex::new(reader)),
      reader,
    }
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueWithRWFactoryBehavior<E> for BlockingQueue<E, Q> {}

// ---

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> QueueWriterBehavior<E> for BlockingQueueWriter<E, Q> {
  fn offer(&mut self, e: E) -> anyhow::Result<()> {
    self.writer.offer(e)
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> QueueBehavior<E> for BlockingQueueWriter<E, Q> {
  fn len(&self) -> QueueSize {
    let q = self.queue.underlying.0.lock().unwrap();
    q.len()
  }

  fn capacity(&self) -> QueueSize {
    let q = self.queue.underlying.0.lock().unwrap();
    q.capacity()
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueWriterBehavior<E> for BlockingQueueWriter<E, Q> {
  fn put(&mut self, e: E) -> anyhow::Result<()> {
    let (queue_mutex, not_full, not_empty) = &*self.queue.underlying;
    let result = {
      log::debug!("put: queue_mutex.lock().unwrap()");
      let mut queue_mutex_guard = queue_mutex.lock().unwrap();
      log::debug!("put: not_full.wait(queue_mutex_guard).unwrap()");
      while queue_mutex_guard.is_full() {
        log::debug!("put: not_full.wait...");
        queue_mutex_guard = not_full.wait(queue_mutex_guard).unwrap();
        log::debug!("put: not_full.wait...done");
      }
      log::debug!("put: self.writer.offer(e)");
      let result = self.writer.offer(e);
      result
    };
    log::debug!("put: not_empty.notify_one()");
    not_empty.notify_one();
    log::debug!("put: result");
    result
  }
}

// ---

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> QueueReaderBehavior<E> for BlockingQueueReader<E, Q> {
  fn poll(&mut self) -> anyhow::Result<Option<E>> {
    self.reader.poll()
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> QueueBehavior<E> for BlockingQueueReader<E, Q> {
  fn len(&self) -> QueueSize {
    let q = self.queue.underlying.0.lock().unwrap();
    q.len()
  }

  fn capacity(&self) -> QueueSize {
    let q = self.queue.underlying.0.lock().unwrap();
    q.capacity()
  }
}

impl<E: Element, Q: QueueWithRWFactoryBehavior<E>> BlockingQueueReaderBehavior<E> for BlockingQueueReader<E, Q> {
  fn take(&mut self) -> anyhow::Result<Option<E>> {
    let (queue_mutex, not_full, not_empty) = &*self.queue.underlying;
    let result = {
      log::debug!("take: queue_mutex.lock().unwrap()");
      let mut queue_mutex_guard = queue_mutex.lock().unwrap();
      log::debug!("take: not_empty.wait(queue_mutex_guard).unwrap()");
      while queue_mutex_guard.is_empty() {
        log::debug!("take: not_empty.wait...");
        queue_mutex_guard = not_empty.wait(queue_mutex_guard).unwrap();
        log::debug!("take: not_empty.wait...done");
      }
      log::debug!("take: self.reader.poll()");
      let result = self.reader.poll();
      result
    };
    log::debug!("take: not_full.notify_one()");
    not_full.notify_one();
    log::debug!("take: result");
    result
  }
}

#[cfg(test)]
mod tests {
  use crate::infrastructure::queue::QueueVec;
  use std::sync::mpsc;
  use std::thread;
  use std::time::Duration;

  use super::*;

  #[test]
  fn test_blocking_queue_basic() {
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(5);
    let blocking_queue = BlockingQueue::new(array_queue);

    assert_eq!(blocking_queue.len(), QueueSize::Limited(0));
    assert_eq!(blocking_queue.capacity(), QueueSize::Limited(5));
    let mut writer = blocking_queue.writer();
    let mut reader = blocking_queue.reader();
    writer.offer(1).unwrap();
    assert_eq!(blocking_queue.len(), QueueSize::Limited(1));

    writer.offer(2).unwrap();
    assert_eq!(blocking_queue.len(), QueueSize::Limited(2));

    let polled = reader.poll().unwrap();
    assert_eq!(polled, Some(1));
    assert_eq!(blocking_queue.len(), QueueSize::Limited(1));

    let polled = reader.poll().unwrap();
    assert_eq!(polled, Some(2));
    assert_eq!(blocking_queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_blocking_queue_put_take() {
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(2);
    let blocking_queue = BlockingQueue::new(array_queue);

    let (tx, rx) = mpsc::channel();
    let cloned_tx = tx.clone();

    let blocking_queue_cloned1 = blocking_queue.writer().clone();
    let producer = thread::spawn(move || {
      for i in 1..=2 {
        blocking_queue_cloned1.clone().put(i).unwrap();
        cloned_tx.send(i).unwrap();
      }
    });

    let blocking_queue_cloned2 = blocking_queue.reader().clone();
    let consumer = thread::spawn(move || {
      for _ in 1..=2 {
        let received = rx.recv().unwrap();
        let taken = blocking_queue_cloned2.clone().take().unwrap();
        assert_eq!(taken, Some(received));
      }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
  }

  #[test]
  fn test_blocking_queue_concurrent() {
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(2);
    let queue = Arc::new(BlockingQueue::new(array_queue));
    let queue_clone = queue.clone();

    let writer_thread = thread::spawn(move || {
      log::debug!("writer_thread start");
      log::debug!("writer_thread: writer()");
      let mut writer = queue_clone.writer();
      for i in 0..10 {
        log::debug!("writer_thread: put({})", i);
        writer.put(i).unwrap();
        log::debug!("writer_thread: i = {}", i);
        thread::sleep(Duration::from_millis(10));
      }
    });

    let reader_thread = thread::spawn(move || {
      log::debug!("reader_thread start");
      log::debug!("reader_thread: reader()");
      let mut reader = queue.reader();
      for _ in 0..10 {
        log::debug!("reader_thread: take(), size = {:?}: start", reader.len());
        let result = reader.take();
        log::debug!("reader_thread: take(): finished: result = {:?}", result);
        if let Ok(Some(msg)) = result {
          log::debug!("msg = {}", msg);
        }
      }
    });

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
  }
}
