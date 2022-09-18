use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::dispatch::message::Message;

#[derive(Debug)]
pub struct DowncastAnyMessageError;

pub struct AnyMessage {
  pub one_time: bool,
  pub msg: Option<Arc<dyn Any + Send>>,
}

impl AnyMessage {
  pub fn new<T>(msg: T, one_time: bool) -> Self
  where
    T: Any + Message + 'static, {
    Self {
      one_time,
      msg: Some(Arc::new(msg)),
    }
  }

  pub fn take<T>(&mut self) -> Result<Arc<T>, DowncastAnyMessageError>
  where
    T: Any + Message + 'static, {
    if self.one_time {
      match self.msg.take() {
        Some(m) => {
          if (*m).is::<T>() {
            let ptr = Arc::into_raw(m).cast::<T>();
            let s = unsafe { Arc::from_raw(ptr) };
            Ok(s)
          } else {
            Err(DowncastAnyMessageError)
          }
        }
        None => Err(DowncastAnyMessageError),
      }
    } else {
      match self.msg.as_ref() {
        Some(m) if (*m).is::<T>() => {
          let ptr = Arc::into_raw(m.clone()).cast::<T>();
          let s = unsafe { Arc::from_raw(ptr) };
          Ok(s)
        }
        Some(_) => Err(DowncastAnyMessageError),
        None => Err(DowncastAnyMessageError),
      }
    }
  }
}

impl Clone for AnyMessage {
  fn clone(&self) -> Self {
    Self {
      one_time: self.one_time,
      msg: self.msg.clone(),
    }
  }
}

impl Debug for AnyMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("AnyMessage")
  }
}

impl PartialEq for AnyMessage {
  fn eq(&self, other: &Self) -> bool {
    self.one_time == other.one_time
      && match (&self.msg, &other.msg) {
        (Some(l), Some(r)) => {
          let lp = Arc::into_raw(l.clone());
          let rp = Arc::into_raw(r.clone());
          lp == rp
        }
        (None, None) => true,
        _ => false,
      }
  }
}
