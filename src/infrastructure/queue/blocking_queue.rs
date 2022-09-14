use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Condvar, Mutex};

use super::*;
use anyhow::Result;

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
