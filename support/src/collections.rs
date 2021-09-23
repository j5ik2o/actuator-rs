use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use std::thread::sleep;

pub trait Queue<E> {
  fn len(&self) -> usize;
  fn offer(&mut self, e: E) -> Result<bool>;
  fn poll(&mut self) -> Option<E>;
  fn peek(&self) -> Option<&E>;
}

pub trait Deque<E>: Queue<E> {
  fn offer_first(&mut self, e: E) -> Result<bool>;
  fn offer_last(&mut self, e: E) -> Result<bool> {
    self.offer(e)
  }

  fn poll_first(&mut self) -> Option<E> {
    self.poll()
  }
  fn poll_last(&mut self) -> Option<E>;

  fn peek_first(&self) -> Option<&E> {
    self.peek()
  }
  fn peek_last(&self) -> Option<&E>;
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
    Self {
      values: vec,
      num_elements,
    }
  }
}

impl<E> Queue<E> for VecQueue<E> {
  fn len(&self) -> usize {
    self.values.len()
  }

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

  fn poll_last(&mut self) -> Option<E> {
    self.values.pop_back()
  }

  fn peek_last(&self) -> Option<&E> {
    self.values.back()
  }
}

#[derive(Debug, Clone)]
pub struct BlockingVecQueue<E> {
  inner: Arc<Mutex<BlockingVecQueueInner<E>>>,
  p: Option<E>,
}

#[derive(Debug)]
struct BlockingVecQueueInner<E> {
  q: VecQueue<E>,
  take_lock: Mutex<bool>,
  take_cvar: Condvar,
  put_lock: Mutex<bool>,
  put_cvar: Condvar,
}

impl<E> BlockingVecQueueInner<E> {
  pub fn new(
    q: VecQueue<E>,
    take_lock: Mutex<bool>,
    take_cvar: Condvar,
    put_lock: Mutex<bool>,
    put_cvar: Condvar,
  ) -> Self {
    Self {
      q,
      take_lock,
      take_cvar,
      put_lock,
      put_cvar,
    }
  }
}

impl<E> BlockingVecQueue<E> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(BlockingVecQueueInner::new(
        VecQueue::new(),
        Mutex::new(false),
        Condvar::new(),
        Mutex::new(false),
        Condvar::new(),
      ))),
      p: None,
    }
  }

  pub fn with_num_elements(num_elements: usize) -> Self {
    Self {
      inner: Arc::new(Mutex::new(BlockingVecQueueInner::new(
        VecQueue::with_num_elements(num_elements),
        Mutex::new(false),
        Condvar::new(),
        Mutex::new(false),
        Condvar::new(),
      ))),
      p: None,
    }
  }
}

impl<E: Clone> Queue<E> for BlockingVecQueue<E> {
  fn len(&self) -> usize {
    let inner = self.inner.lock().unwrap();
    let lq = &inner.q;
    lq.len()
  }

  fn offer(&mut self, e: E) -> Result<bool> {
    let mut inner = self.inner.lock().unwrap();
    let lq = &mut inner.q;
    lq.offer(e)
  }

  fn poll(&mut self) -> Option<E> {
    let mut inner = self.inner.lock().unwrap();
    let lq = &mut inner.q;
    let result = lq.poll();
    self.p = lq.peek().map(|e| e.clone());
    result
  }

  fn peek(&self) -> Option<&E> {
    self.p.as_ref()
  }
}

impl<E: Clone> Deque<E> for BlockingVecQueue<E> {
  fn offer_first(&mut self, e: E) -> Result<bool> {
    let mut inner = self.inner.lock().unwrap();
    inner.q.offer_first(e)
  }

  fn poll_last(&mut self) -> Option<E> {
    let mut inner = self.inner.lock().unwrap();
    inner.q.poll_last()
  }

  fn peek_last(&self) -> Option<&E> {
    self.p.as_ref()
  }
}
/// https://github.com/JimFawcett/RustBlockingQueue/blob/master/src/lib.rs
/// https://docs.rs/fp_rust/0.1.39/src/fp_rust/sync.rs.html#300

impl<E: Clone> BlockingQueue<E> for BlockingVecQueue<E> {
  fn put(&mut self, e: E) -> Result<()> {
    loop {
      let inner = self.inner.lock().unwrap();
      log::debug!(
        "len = {}, num_elements = {}",
        inner.q.len(),
        inner.q.num_elements
      );
      if inner.q.len() < inner.q.num_elements {
        break;
      }
      {
        let mut pl = inner.put_lock.lock().unwrap();
        log::debug!("put_cvar#wait..");
        inner
          .put_cvar
          .wait_timeout_while(pl, Duration::from_secs(1), |pl| !*pl)
          .unwrap()
          .0;
        drop(inner);
        sleep(Duration::from_secs(1));
      }
    }
    let mut inner = self.inner.lock().unwrap();
    inner.q.offer(e);
    log::debug!("start: take_cvar#notify_one");
    let mut pl = inner.put_lock.lock().unwrap();
    *pl = true;
    inner.take_cvar.notify_one();
    log::debug!("finish: take_cvar#notify_one");
    Ok(())
  }

  fn take(&mut self) -> Option<E> {
    loop {
      let inner = self.inner.lock().unwrap();
      log::debug!(
        "len = {}, num_elements = {}",
        inner.q.len(),
        inner.q.num_elements
      );
      if inner.q.len() > 0 {
        break;
      }
      {
        let mut tl = inner.take_lock.lock().unwrap();
        log::debug!("take_cvar#wait..");
        inner
          .take_cvar
          .wait_timeout_while(tl, Duration::from_secs(1), |tl| !*tl)
          .unwrap()
          .0;
        drop(inner);
        sleep(Duration::from_secs(1));
      }
    }
    let mut inner = self.inner.lock().unwrap();
    let result = inner.q.poll();
    log::debug!("start: put_cvar#notify_one");
    let mut l = inner.put_lock.lock().unwrap();
    *l = true;
    inner.put_cvar.notify_one();
    log::debug!("finish: put_cvar#notify_one");
    result
  }
}

#[cfg(test)]
mod tests {
  use std::env;
  use std::sync::{Arc, Mutex};
  use std::thread::sleep;
  use std::time::Duration;

  use crate::collections::{BlockingQueue, BlockingVecQueue, Deque, Queue, VecQueue};

  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    // env::set_var("RUST_LOG", "trace");
    let _ = logger::try_init();
  }

  #[test]
  fn test_queue_1() {
    init_logger();

    let mut vec_queue = VecQueue::<u32>::new();

    let result = vec_queue.offer(1).unwrap();
    assert!(result);
    let result = vec_queue.offer(2).unwrap();
    assert!(result);
    let result = vec_queue.offer(3).unwrap();
    assert!(result);

    let result = vec_queue.offer(10).unwrap();
    assert!(result);

    log::debug!("vec_queue = {:?}", vec_queue);

    let peek = vec_queue.peek().unwrap();
    log::debug!("{}", peek);
    let peek = vec_queue.peek_first().unwrap();
    log::debug!("{}", peek);

    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 1);
    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 2);
    let result = vec_queue.poll().unwrap();
    assert_eq!(result, 3);
  }

  #[test]
  fn test_queue_2() {
    init_logger();

    let mut vec_queue = VecQueue::<u32>::with_num_elements(2);

    let result = vec_queue.offer(1).unwrap();
    assert!(result);
    let result = vec_queue.offer(2).unwrap();
    assert!(result);
    let result = vec_queue.offer(3);
    assert!(result.is_err());
  }

  #[test]
  fn test_bq() {
    init_logger();

    let mut bq1 = BlockingVecQueue::<i32>::with_num_elements(1);
    let mut bq2 = bq1.clone();

    use std::thread;

    let handler = thread::spawn(move || {
      log::debug!("take: start: sleep");
      sleep(Duration::from_secs(3));
      log::debug!("take: finish: sleep");

      log::debug!("take: start: take");
      let n = bq2.take();
      log::debug!("take: finish: take");
      log::debug!("take: n = {:?}", n);
    });

    log::debug!("put: start: put - 1");
    bq1.put(1);
    log::debug!("put: finish: put - 1");

    log::debug!("put: start: put - 2");
    bq1.put(1);
    log::debug!("put: finish: put - 2");

    handler.join().unwrap();
  }
}
