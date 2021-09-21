use anyhow::Result;
use anyhow::anyhow;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{Sender, Receiver, channel};
use owning_ref::MutexGuardRef;
use std::ops::Deref;

pub trait Queue<E> {
  fn offer(&mut self, e: E) -> Result<bool>;
  fn poll(&mut self) -> Option<E>;
  fn peek(&mut self) -> Option<&E>;
}

pub trait Deque<E>: Queue<E> {
  fn offer_first(&mut self, e: E) -> Result<bool>;
  fn offer_last(&mut self, e: E) -> Result<bool>;

  fn poll_first(&mut self) -> Option<E>;
  fn poll_last(&mut self) -> Option<E>;

  fn peek_first(&mut self) -> Option<&E>;
  fn peek_last(&mut self) -> Option<&E>;
}

pub trait BlockingQueue<E>: Queue<E> {
  fn put(&mut self, e: E) -> Result<()>;
  fn take(&mut self) -> Option<E>;
}

#[derive(Debug, Clone)]
pub struct VecQueue<E> {
  values: VecDeque<E>,
  num_elements: usize,
}

impl<E> VecQueue<E> {
  fn new() -> Self {
    Self {
      values: VecDeque::new(),
      num_elements: usize::MAX,
    }
  }
  fn with_num_elements(num_elements: usize) -> Self {
    Self {
      values: VecDeque::new(),
      num_elements,
    }
  }
  fn with_elements(values: impl IntoIterator<Item = E> + ExactSizeIterator) -> Self {
    let num_elements = values.len();
    let vec = values.into_iter().collect::<VecDeque<E>>();
    Self { values: vec, num_elements }
  }
}

impl<E> Queue<E> for VecQueue<E> {
  fn offer(&mut self, e: E) -> Result<bool> {
    if self.num_elements >= self.values.len() + 1 {
      self.values.push_back(e);
      Ok(true)
    } else {
      Err(anyhow!(""))
    }
  }

  fn poll(&mut self) -> Option<E> {
    self.values.pop_front()
  }

  fn peek(&mut self) -> Option<&E> {
    self.values.front()
  }
}

impl<E> Deque<E> for VecQueue<E> {
  fn offer_first(&mut self, e: E) -> Result<bool> {
    if self.num_elements >= self.values.len() + 1 {
      self.values.push_front(e);
      Ok(true)
    } else {
      Err(anyhow!(""))
    }
  }

  fn offer_last(&mut self, e: E) -> Result<bool> {
    self.offer(e)
  }

  fn poll_first(&mut self) -> Option<E> {
    self.poll()
  }

  fn poll_last(&mut self) -> Option<E> {
    self.values.pop_back()
  }

  fn peek_first(&mut self) -> Option<&E> {
    self.peek()
  }

  fn peek_last(&mut self) -> Option<&E> {
    self.values.back()
  }
}

#[derive(Debug)]
pub struct BlockingVecQueue<E> {
  q: Mutex<VecQueue<E>>,
  cv: Condvar,
  peek: Option<E>,
}

impl<E> BlockingVecQueue<E> {
  pub fn new() -> Self {
    Self {
      q: Mutex::new(VecQueue::new()),
      cv: Condvar::new(),
      peek: None
    }
  }
}

impl<E: Clone> Queue<E> for BlockingVecQueue<E> {
  fn offer(&mut self, e: E) -> Result<bool> {
    let mut lq = self.q.lock().unwrap();
    lq.offer(e)
  }

  fn poll(&mut self) -> Option<E> {
    let mut lq = self.q.lock().unwrap();
    lq.poll()
  }

  fn peek(&mut self) -> Option<&E> {
    let mut lq = self.q.lock().unwrap();
    self.peek = lq.peek().map(|e| e.clone());
    self.peek.as_ref()
  }
}

impl<E: Clone> BlockingQueue<E> for BlockingVecQueue<E> {
  fn put(&mut self, e: E) -> Result<()> {
    let mut lq = self.q.lock().unwrap();
    lq.offer(e);
    self.cv.notify_one();
    Ok(())
  }

  fn take(&mut self) -> Option<E> {
    let mut lq = self.q.lock().unwrap();
    while lq.values.len() == 0 {
      lq = self.cv.wait(lq).unwrap();
    }
    lq.poll()
  }
}

#[cfg(test)]
mod tests {
  use crate::collections::{VecQueue, Queue, Deque, BlockingVecQueue, BlockingQueue};
  use std::sync::{Arc, Mutex};

  #[test]
  fn test_queue_1() {
    let mut vec_queue = VecQueue::<u32>::new();

    let result = vec_queue.offer(1).unwrap();
    assert!(result);
    let result = vec_queue.offer(2).unwrap();
    assert!(result);
    let result = vec_queue.offer(3).unwrap();
    assert!(result);

    let result = vec_queue.offer(10).unwrap();
    assert!(result);

    println!("vec_queue = {:?}", vec_queue);

    let peek = vec_queue.peek().unwrap();
    println!("{}", peek);
    let peek = vec_queue.peek_first().unwrap();
    println!("{}", peek);

    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 1);
    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 2);
    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 3);

  }

  #[test]
  fn test_queue_2() {
    let mut vec_queue = VecQueue::<u32>::with_num_elements(2);

    let result = vec_queue.offer(1).unwrap();
    assert!(result);
    let result = vec_queue.offer(2).unwrap();
    assert!(result);
    let result = vec_queue.offer(3);
    assert!(result.is_err());

  }

  #[test]
  fn test_bq(){
    let mut bq1 = Arc::new(Mutex::new(BlockingVecQueue::new()));
    let mut bq2 = bq1.clone();

    use std::thread;

    let handler = thread::spawn(move || {
      println!("start: lock");
      let mut bq2_g = bq2.lock().unwrap();
      println!("finish: lock");
      let n = bq2_g.take();
      println!("n = {:?}", n);
    });
    {
      let mut bq1_g = bq1.lock().unwrap();
      bq1_g.put(1);
    }
    handler.join().unwrap();
  }
}