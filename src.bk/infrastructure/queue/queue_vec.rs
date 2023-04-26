use crate::infrastructure::queue::{
  Element, QueueBehavior, QueueError, QueueRWFactoryBehavior, QueueReaderBehavior, QueueReaderFactoryBehavior,
  QueueSize, QueueWithRWFactoryBehavior, QueueWriterBehavior, QueueWriterFactoryBehavior,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct QueueVec<E: Element> {
  values: Arc<Mutex<VecDeque<E>>>,
  pub(crate) capacity: QueueSize,
}

unsafe impl<E: Element> Send for QueueVec<E> {}
unsafe impl<E: Element> Sync for QueueVec<E> {}

impl<E: Element + PartialEq> PartialEq for QueueVec<E> {
  fn eq(&self, other: &Self) -> bool {
    let l = self.values.lock().unwrap();
    let r = other.values.lock().unwrap();
    &*l == &*r
  }
}

#[derive(Debug, Clone)]
pub struct QueueVecWriter<E: Element> {
  queue: QueueVec<E>,
}

unsafe impl<E: Element> Send for QueueVecWriter<E> {}
unsafe impl<E: Element> Sync for QueueVecWriter<E> {}

#[derive(Debug, Clone)]
pub struct QueueVecReader<E: Element> {
  queue: QueueVec<E>,
}

unsafe impl<E: Element> Send for QueueVecReader<E> {}
unsafe impl<E: Element> Sync for QueueVecReader<E> {}

impl<E: Element> QueueVec<E> {
  pub fn new() -> Self {
    Self {
      values: Arc::new(Mutex::new(VecDeque::new())),
      capacity: QueueSize::Limitless,
    }
  }

  pub fn with_num_elements(num_elements: usize) -> Self {
    Self {
      values: Arc::new(Mutex::new(VecDeque::new())),
      capacity: QueueSize::Limited(num_elements),
    }
  }

  pub fn with_elements(values: impl IntoIterator<Item = E> + ExactSizeIterator) -> Self {
    let num_elements = values.len();
    let vec = values.into_iter().collect::<VecDeque<E>>();
    Self {
      values: Arc::new(Mutex::new(vec)),
      capacity: QueueSize::Limited(num_elements),
    }
  }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueVec<E> {
  fn len(&self) -> QueueSize {
    let mg = self.values.lock().unwrap();
    let len = mg.len();
    QueueSize::Limited(len)
  }

  fn capacity(&self) -> QueueSize {
    self.capacity.clone()
  }
}

impl<E: Element + 'static> QueueWriterFactoryBehavior<E> for QueueVec<E> {
  type Writer = QueueVecWriter<E>;

  fn writer(&self) -> Self::Writer {
    QueueVecWriter { queue: self.clone() }
  }
}

impl<E: Element + 'static> QueueReaderFactoryBehavior<E> for QueueVec<E> {
  type Reader = QueueVecReader<E>;

  fn reader(&self) -> Self::Reader {
    QueueVecReader { queue: self.clone() }
  }
}

impl<E: Element + 'static> QueueRWFactoryBehavior<E> for QueueVec<E> {}

impl<E: Element + 'static> QueueWithRWFactoryBehavior<E> for QueueVec<E> {}

impl<E: Element + 'static> QueueBehavior<E> for QueueVecWriter<E> {
  fn len(&self) -> QueueSize {
    self.queue.len()
  }

  fn capacity(&self) -> QueueSize {
    self.queue.capacity()
  }
}

impl<E: Element + 'static> QueueWriterBehavior<E> for QueueVecWriter<E> {
  fn offer(&mut self, e: E) -> anyhow::Result<()> {
    if self.non_full() {
      let mut mg = self.queue.values.lock().unwrap();
      mg.push_back(e);
      Ok(())
    } else {
      Err(anyhow::Error::new(QueueError::OfferError(e)))
    }
  }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueVecReader<E> {
  fn len(&self) -> QueueSize {
    self.queue.len()
  }

  fn capacity(&self) -> QueueSize {
    self.queue.capacity()
  }
}

impl<E: Element + 'static> QueueReaderBehavior<E> for QueueVecReader<E> {
  fn poll(&mut self) -> anyhow::Result<Option<E>> {
    let mut mg = self.queue.values.lock().unwrap();
    Ok(mg.pop_front())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  fn init_logger() {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_queue_vec_new() {
    init_logger();
    let queue = QueueVec::<i32>::new();

    assert_eq!(queue.capacity(), QueueSize::Limitless);
    assert_eq!(queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_queue_vec_with_num_elements() {
    init_logger();
    let num_elements = 5;
    let queue = QueueVec::<i32>::with_num_elements(num_elements);

    assert_eq!(queue.capacity(), QueueSize::Limited(num_elements));
    assert_eq!(queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_queue_vec_with_elements() {
    init_logger();
    let elements = vec![1, 2, 3];
    let queue = QueueVec::<i32>::with_elements(elements.into_iter());

    assert_eq!(queue.capacity(), QueueSize::Limited(3));
    assert_eq!(queue.len(), QueueSize::Limited(3));

    let mut reader = queue.reader();
    assert_eq!(reader.poll().unwrap().unwrap(), 1);
    assert_eq!(reader.len(), QueueSize::Limited(2));
  }

  #[test]
  fn test_queue_vec_offer() {
    init_logger();
    let queue = QueueVec::<i32>::new();

    assert_eq!(queue.capacity(), QueueSize::Limitless);
    assert_eq!(queue.len(), QueueSize::Limited(0));

    let mut writer = queue.writer();
    writer.offer(1).unwrap();
    writer.offer(2).unwrap();

    assert_eq!(writer.len(), QueueSize::Limited(2));

    let mut reader = queue.reader();
    assert_eq!(reader.poll().unwrap().unwrap(), 1);
    assert_eq!(reader.len(), QueueSize::Limited(1));
  }
}
