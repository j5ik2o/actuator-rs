use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Condvar, Mutex};

use anyhow::Result;

use super::*;

#[derive(Debug, Clone)]
pub struct BlockingQueue<E, Q: QueueBehavior<E>> {
  underlying: Arc<(Mutex<Q>, Condvar, Condvar)>,
  p: PhantomData<E>,
}

impl<E: Element + 'static, Q: QueueBehavior<E>> QueueBehavior<E> for BlockingQueue<E, Q> {
  fn len(&self) -> QueueSize {
    let (queue_vec_mutex, _, _) = &*self.underlying;
    let queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    queue_vec_mutex_guard.len()
  }

  fn capacity(&self) -> QueueSize {
    let (queue_vec_mutex, _, _) = &*self.underlying;
    let queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    queue_vec_mutex_guard.capacity()
  }

  fn offer(&mut self, e: E) -> Result<()> {
    let (queue_vec_mutex, _, not_empty) = &*self.underlying;
    let mut queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    let result = queue_vec_mutex_guard.offer(e);
    not_empty.notify_one();
    result
  }

  fn poll(&mut self) -> Result<Option<E>> {
    let (queue_vec_mutex, not_full, _) = &*self.underlying;
    let mut queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    let result = queue_vec_mutex_guard.poll();
    not_full.notify_one();
    result
  }

  // fn peek(&self) -> Result<Option<E>> {
  //   let (queue_vec_mutex, not_full, _) = &*self.underlying;
  //   let queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
  //   let result = queue_vec_mutex_guard.peek();
  //   not_full.notify_one();
  //   result
  // }
}

impl<E: Element + 'static, Q: QueueBehavior<E>> BlockingQueueBehavior<E> for BlockingQueue<E, Q> {
  fn put(&mut self, e: E) -> Result<()> {
    let (queue_vec_mutex, not_full, not_empty) = &*self.underlying;
    let mut queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    while queue_vec_mutex_guard.is_full() {
      queue_vec_mutex_guard = not_full.wait(queue_vec_mutex_guard).unwrap();
    }
    let result = queue_vec_mutex_guard.offer(e);
    not_empty.notify_one();
    result
  }

  fn take(&mut self) -> Result<Option<E>> {
    let (queue_vec_mutex, not_full, not_empty) = &*self.underlying;
    let mut queue_vec_mutex_guard = queue_vec_mutex.lock().unwrap();
    while queue_vec_mutex_guard.is_empty() {
      queue_vec_mutex_guard = not_empty.wait(queue_vec_mutex_guard).unwrap();
    }
    let result = queue_vec_mutex_guard.poll();
    not_full.notify_one();
    result
  }
}

impl<E, Q: QueueBehavior<E>> BlockingQueue<E, Q> {
  pub fn new(queue: Q) -> Self {
    Self {
      underlying: Arc::new((Mutex::new(queue), Condvar::new(), Condvar::new())),
      p: PhantomData::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::mpsc;
  use std::thread;

  use super::*;

  #[test]
  fn test_blocking_queue_basic() {
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(5);
    let mut blocking_queue = BlockingQueue::new(array_queue);

    assert_eq!(blocking_queue.len(), QueueSize::Limited(0));
    assert_eq!(blocking_queue.capacity(), QueueSize::Limited(5));

    blocking_queue.offer(1).unwrap();
    assert_eq!(blocking_queue.len(), QueueSize::Limited(1));

    blocking_queue.offer(2).unwrap();
    assert_eq!(blocking_queue.len(), QueueSize::Limited(2));

    let polled = blocking_queue.poll().unwrap();
    assert_eq!(polled, Some(1));
    assert_eq!(blocking_queue.len(), QueueSize::Limited(1));

    let polled = blocking_queue.poll().unwrap();
    assert_eq!(polled, Some(2));
    assert_eq!(blocking_queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_blocking_queue_put_take() {
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(2);
    let blocking_queue = BlockingQueue::new(array_queue);

    let (tx, rx) = mpsc::channel();
    let cloned_tx = tx.clone();

    let blocking_queue_cloned1 = blocking_queue.clone();
    let producer = thread::spawn(move || {
      for i in 1..=2 {
        blocking_queue_cloned1.clone().put(i).unwrap();
        cloned_tx.send(i).unwrap();
      }
    });

    let blocking_queue_cloned2 = blocking_queue.clone();
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
    let array_queue: QueueVec<u32> = QueueVec::with_num_elements(5);
    let blocking_queue = Arc::new(Mutex::new(BlockingQueue::new(array_queue)));

    let mut handles = vec![];

    for i in 1..=5 {
      let blocking_queue = Arc::clone(&blocking_queue);
      let handle = thread::spawn(move || {
        let mut blocking_queue = blocking_queue.lock().unwrap();
        blocking_queue.put(i).unwrap();
      });
      handles.push(handle);
    }

    for _ in 1..=5 {
      let blocking_queue = Arc::clone(&blocking_queue);
      let handle = thread::spawn(move || {
        let mut blocking_queue = blocking_queue.lock().unwrap();
        let taken = blocking_queue.take().unwrap();
        assert!(taken.is_some());
      });
      handles.push(handle);
    }

    for handle in handles {
      handle.join().unwrap();
    }
  }
}
