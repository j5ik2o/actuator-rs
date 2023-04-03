use crate::core::dispatch::system_message::system_message::SystemMessage;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SystemMessageEntry {
  pub message: SystemMessage,
  pub next: Option<Arc<Mutex<SystemMessageEntry>>>,
}

unsafe impl Send for SystemMessageEntry {}
unsafe impl Sync for SystemMessageEntry {}

impl PartialEq for SystemMessageEntry {
  fn eq(&self, other: &Self) -> bool {
    self.message == other.message
      && match (self.next.as_ref(), other.next.as_ref()) {
        (Some(v1_arc), Some(v2_arc)) => {
          if Arc::ptr_eq(v1_arc, v2_arc) {
            true
          } else {
            let v1_guard = v1_arc.lock().unwrap();
            let v2_guard = v2_arc.lock().unwrap();
            &*v1_guard == &*v2_guard
          }
        }
        (None, None) => true,
        _ => false,
      }
  }
}

impl SystemMessageEntry {
  pub fn new(message: SystemMessage) -> Self {
    SystemMessageEntry { message, next: None }
  }

  pub fn new_with_next(message: SystemMessage, next: Option<Arc<Mutex<SystemMessageEntry>>>) -> Self {
    SystemMessageEntry { message, next }
  }

  pub fn set_next(&mut self, next: Option<Arc<Mutex<SystemMessageEntry>>>) {
    self.next = next;
  }

  pub fn unlink(&mut self) {
    self.set_next(None);
  }

  pub fn is_unlinked(&self) -> bool {
    self.next.is_none()
  }

  pub fn is_no_message(&self) -> bool {
    self.message.is_no_message()
  }
}
