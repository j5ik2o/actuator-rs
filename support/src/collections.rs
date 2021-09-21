use anyhow::Result;
use anyhow::anyhow;
use std::collections::VecDeque;

pub trait Queue<E> {
  fn offer(&mut self, e: E) -> Result<bool>;
  fn poll(&mut self) -> Option<E>;
  fn peek(&self) -> Option<&E>;
}

pub trait Deque<E>: Queue<E> {
  fn offer_first(&mut self, e: E) -> Result<bool>;
  fn offer_last(&mut self, e: E) -> Result<bool>;

  fn poll_first(&mut self) -> Option<E>;
  fn poll_last(&mut self) -> Option<E>;

  fn peek_first(&self) -> Option<&E>;
  fn peek_last(&self) -> Option<&E>;
}

pub trait BlockingQueue<E>: Queue<E> {
  fn put(&mut self, e: E);
  fn take(&mut self) -> Result<E>;
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

  fn peek(&self) -> Option<&E> {
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

  fn peek_first(&self) -> Option<&E> {
    self.peek()
  }

  fn peek_last(&self) -> Option<&E> {
    self.values.back()
  }
}

#[cfg(test)]
mod tests {
  use crate::collections::{VecQueue, Queue, Deque};

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
}