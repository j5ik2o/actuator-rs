use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use crate::dispatch::message::Message;

#[derive(Debug)]
pub struct DowncastAnyMessageError;

pub struct AnyMessage {
  pub one_time: bool,
  pub msg: Option<Rc<dyn Any + Send>>,
}

unsafe impl Send for AnyMessage {}
unsafe impl Sync for AnyMessage {}

impl AnyMessage {
  pub fn new<T>(msg: T) -> Self
  where
    T: Any + Message + 'static, {
    Self::new_with(msg, false)
  }

  pub fn new_with<T>(msg: T, one_time: bool) -> Self
  where
    T: Any + Message + 'static, {
    Self {
      one_time,
      msg: Some(Rc::new(msg)),
    }
  }

  pub fn is_type<T>(&self) -> bool
  where
    T: Any + Message + 'static, {
    match self.msg.as_ref() {
      Some(m) => m.is::<T>(),
      None => false,
    }
  }

  pub fn has_message(&self) -> bool {
    self.msg.is_some()
  }

  // pub fn is_system_message<Msg: Message>(&self) -> bool {
  //   self.is_type::<SystemMessage<Msg>>()
  // }

  pub fn take_once<T>(&mut self) -> Result<T, DowncastAnyMessageError>
  where
    T: Any + Message + 'static, {
    if self.one_time {
      match self.msg.take() {
        Some(m) if (*m).is::<T>() => {
          let ptr = Rc::into_raw(m).cast::<T>();
          let s = unsafe { Rc::from_raw(ptr) };
          Ok((&*s).clone())
        }
        Some(_) => Err(DowncastAnyMessageError),
        None => Err(DowncastAnyMessageError),
      }
    } else {
      self.take()
    }
  }

  pub fn take<T>(&self) -> Result<T, DowncastAnyMessageError>
  where
    T: Any + Message + 'static, {
    if self.one_time {
      panic!("Can't take one time message");
    } else {
      match self.msg.as_ref() {
        Some(m) if (*m).is::<T>() => {
          let ptr = Rc::into_raw(m.clone()).cast::<T>();
          let s = unsafe { Rc::from_raw(ptr) };
          Ok((&*s).clone())
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
    write!(f, "AnyMessage {{ msg: {:?}, one_time: {} }}", self.msg, self.one_time)
  }
}

impl PartialEq for AnyMessage {
  fn eq(&self, other: &Self) -> bool {
    self.one_time == other.one_time
      && match (&self.msg, &other.msg) {
        (Some(l), Some(r)) => {
          let lp = Rc::into_raw(l.clone());
          let rp = Rc::into_raw(r.clone());
          lp == rp
        }
        (None, None) => true,
        _ => false,
      }
  }
}
