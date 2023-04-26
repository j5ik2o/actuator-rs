use crate::infrastructure::queue::{
  Element, QueueBehavior, QueueError, QueueReaderBehavior, QueueReaderFactoryBehavior, QueueSize, QueueWriterBehavior,
  QueueWriterFactoryBehavior,
};
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct QueueMPSC<E: Element> {
  rx: Arc<Mutex<Receiver<E>>>,
  tx: Sender<E>,
  count: Arc<Mutex<QueueSize>>,
  capacity: Arc<Mutex<QueueSize>>,
}

unsafe impl<E: Element> Send for QueueMPSC<E> {}
unsafe impl<E: Element> Sync for QueueMPSC<E> {}

impl<E: Element> PartialEq for QueueMPSC<E> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.rx, &other.rx)
      && Arc::ptr_eq(&self.count, &other.count)
      && Arc::ptr_eq(&self.capacity, &other.capacity)
  }
}

#[derive(Debug, Clone)]
pub struct QueueMPSCWriter<E: Element> {
  queue: QueueMPSC<E>,
}

unsafe impl<E: Element> Send for QueueMPSCWriter<E> {}
unsafe impl<E: Element> Sync for QueueMPSCWriter<E> {}

#[derive(Debug, Clone)]
pub struct QueueMPSCReader<E: Element> {
  queue: QueueMPSC<E>,
}

unsafe impl<E: Element> Send for QueueMPSCReader<E> {}
unsafe impl<E: Element> Sync for QueueMPSCReader<E> {}

impl<E: Element + 'static> QueueMPSC<E> {
  pub fn new() -> Self {
    let (tx, rx) = channel();
    Self {
      rx: Arc::new(Mutex::new(rx)),
      tx,
      count: Arc::new(Mutex::new(QueueSize::Limited(0))),
      capacity: Arc::new(Mutex::new(QueueSize::Limitless)),
    }
  }

  pub fn with_num_elements(num_elements: usize) -> Self {
    let (tx, rx) = channel();
    Self {
      rx: Arc::new(Mutex::new(rx)),
      tx,
      count: Arc::new(Mutex::new(QueueSize::Limited(0))),
      capacity: Arc::new(Mutex::new(QueueSize::Limited(num_elements))),
    }
  }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueMPSC<E> {
  fn len(&self) -> QueueSize {
    let count_guard = self.count.lock().unwrap();
    count_guard.clone()
  }

  fn capacity(&self) -> QueueSize {
    let capacity_guard = self.capacity.lock().unwrap();
    capacity_guard.clone()
  }
}

impl<E: Element + 'static> QueueWriterFactoryBehavior<E> for QueueMPSC<E> {
  type Writer = QueueMPSCWriter<E>;

  fn writer(&self) -> Self::Writer {
    QueueMPSCWriter { queue: self.clone() }
  }
}

impl<E: Element + 'static> QueueReaderFactoryBehavior<E> for QueueMPSC<E> {
  type Reader = QueueMPSCReader<E>;

  fn reader(&self) -> Self::Reader {
    QueueMPSCReader { queue: self.clone() }
  }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueMPSCReader<E> {
  fn len(&self) -> QueueSize {
    self.queue.len()
  }

  fn capacity(&self) -> QueueSize {
    self.queue.capacity()
  }
}

impl<E: Element + 'static> QueueReaderBehavior<E> for QueueMPSCReader<E> {
  fn poll(&mut self) -> anyhow::Result<Option<E>> {
    let receiver_guard = self.queue.rx.lock().unwrap();
    match receiver_guard.try_recv() {
      Ok(e) => {
        let mut count_guard = self.queue.count.lock().unwrap();
        count_guard.decrement();
        Ok(Some(e))
      }
      Err(TryRecvError::Empty) => Ok(None),
      Err(TryRecvError::Disconnected) => Err(anyhow::Error::new(QueueError::<E>::PoolError)),
    }
  }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueMPSCWriter<E> {
  fn len(&self) -> QueueSize {
    self.queue.len()
  }

  fn capacity(&self) -> QueueSize {
    self.queue.capacity()
  }
}

impl<E: Element + 'static> QueueWriterBehavior<E> for QueueMPSCWriter<E> {
  fn offer(&mut self, e: E) -> anyhow::Result<()> {
    match self.queue.tx.send(e) {
      Ok(_) => {
        let mut count_guard = self.queue.count.lock().unwrap();
        count_guard.increment();
        Ok(())
      }
      Err(SendError(e)) => Err(anyhow::Error::new(QueueError::OfferError(e))),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  fn init_logger() {
    let _ = env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_queue_mpsc_new() {
    init_logger();
    let queue = QueueMPSC::<i32>::new();

    assert_eq!(queue.capacity(), QueueSize::Limitless);
    assert_eq!(queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_queue_mpsc_with_num_elements() {
    init_logger();
    let num_elements = 5;
    let queue = QueueMPSC::<i32>::with_num_elements(num_elements);

    assert_eq!(queue.capacity(), QueueSize::Limited(num_elements));
    assert_eq!(queue.len(), QueueSize::Limited(0));
  }

  #[test]
  fn test_queue_mpsc_offer() {
    init_logger();
    let queue = QueueMPSC::<i32>::new();

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
