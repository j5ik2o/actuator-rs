use std::sync::{Arc, Mutex};

use crate::core::dispatch::system_message::earliest_first_system_message_list::EarliestFirstSystemMessageList;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::system_message_list::SystemMessageList;
use crate::core::dispatch::system_message::{reverse_inner, size_inner};

#[derive(Debug, Clone)]
pub struct LatestFirstSystemMessageList {
  pub(crate) head: Option<Arc<Mutex<SystemMessageEntry>>>,
}

unsafe impl Sync for LatestFirstSystemMessageList {}

impl PartialEq for LatestFirstSystemMessageList {
  fn eq(&self, other: &Self) -> bool {
    match (&self.head, &other.head) {
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
impl LatestFirstSystemMessageList {
  pub fn new(head_opt: Option<SystemMessageEntry>) -> Self {
    match head_opt {
      Some(head) => Self {
        head: Some(Arc::new(Mutex::new(head))),
      },
      None => Self { head: None },
    }
  }
}

impl SystemMessageList for LatestFirstSystemMessageList {
  type Other = EarliestFirstSystemMessageList;

  fn is_empty(&self) -> bool {
    self.head.is_none()
  }

  fn size(&self) -> usize {
    size_inner(self.head.clone(), 0)
  }

  fn head(&self) -> Option<&Arc<Mutex<SystemMessageEntry>>> {
    self.head.as_ref()
  }

  fn set_head(&mut self, value: Option<Arc<Mutex<SystemMessageEntry>>>) {
    self.head = value;
  }

  fn tail(&self) -> LatestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|system_message_arc| {
      let system_message_guard = system_message_arc.lock().unwrap();
      system_message_guard.next.clone()
    });
    LatestFirstSystemMessageList { head: next }
  }

  fn prepend(self, mut msg: SystemMessageEntry) -> LatestFirstSystemMessageList {
    msg.set_next(self.head);
    LatestFirstSystemMessageList {
      head: Some(Arc::new(Mutex::new(msg))),
    }
  }

  fn reverse(self) -> Self::Other {
    EarliestFirstSystemMessageList {
      head: reverse_inner(self.head, None),
    }
  }
}
