use std::any::Any;
use std::fmt;
use std::fmt::Debug;

use crate::dispatch::message::Message;

pub struct DowncastAnyMessageError;

pub struct AnyMessage {
  pub one_time: bool,
  pub msg: Option<Box<dyn Any + Send>>,
}

impl AnyMessage {
  pub fn new<T>(msg: T, one_time: bool) -> Self
  where
    T: Any + Message, {
    Self {
      one_time,
      msg: Some(Box::new(msg)),
    }
  }

  pub fn take<T>(&mut self) -> Result<T, DowncastAnyMessageError>
  where
    T: Any + Message, {
    if self.one_time {
      match self.msg.take() {
        Some(m) => {
          if m.is::<T>() {
            Ok(*m.downcast::<T>().unwrap())
          } else {
            Err(DowncastAnyMessageError)
          }
        }
        None => Err(DowncastAnyMessageError),
      }
    } else {
      match self.msg.as_ref() {
        Some(m) if m.is::<T>() => Ok(m.downcast_ref::<T>().cloned().unwrap()),
        Some(_) => Err(DowncastAnyMessageError),
        None => Err(DowncastAnyMessageError),
      }
    }
  }
}

impl Clone for AnyMessage {
  fn clone(&self) -> Self {
    panic!("Can't clone a message of type `AnyMessage`");
  }
}

impl Debug for AnyMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("AnyMessage")
  }
}
