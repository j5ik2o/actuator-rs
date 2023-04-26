use std::sync::{Arc, Mutex};

use crate::core::dispatch::system_message::SystemMessageEntry;

pub trait SystemMessageList {
  type Other: SystemMessageList;

  fn is_empty(&self) -> bool;
  fn non_empty(&self) -> bool {
    !self.is_empty()
  }
  fn size(&self) -> usize;
  fn head(&self) -> Option<&Arc<Mutex<SystemMessageEntry>>>;
  fn set_head(&mut self, value: Option<Arc<Mutex<SystemMessageEntry>>>);
  fn tail(&self) -> Self;
  fn prepend(self, msg: SystemMessageEntry) -> Self;

  fn reverse(self) -> Self::Other;
}
