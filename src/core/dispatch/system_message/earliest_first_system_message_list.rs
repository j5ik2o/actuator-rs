use std::sync::{Arc, Mutex};

use crate::core::dispatch::system_message::latest_first_system_message_list::LatestFirstSystemMessageList;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::system_message_list::SystemMessageList;
use crate::core::dispatch::system_message::{reverse_inner, size_inner};

#[derive(Debug, Clone)]
pub struct EarliestFirstSystemMessageList {
  pub(crate) head: Option<Arc<Mutex<SystemMessageEntry>>>,
}

impl PartialEq for EarliestFirstSystemMessageList {
  fn eq(&self, other: &Self) -> bool {
    match (&self.head, &other.head) {
      (Some(v1), Some(v2)) => {
        if (v1.as_ref() as *const _) == (v2.as_ref() as *const _) {
          true
        } else {
          let v1_inner = v1.lock().unwrap();
          let v2_inner = v2.lock().unwrap();
          &*v1_inner == &*v2_inner
        }
      }
      (None, None) => true,
      _ => false,
    }
  }
}

impl EarliestFirstSystemMessageList {
  pub fn new(head_opt: Option<SystemMessageEntry>) -> Self {
    match head_opt {
      Some(head) => Self {
        head: Some(Arc::new(Mutex::new(head))),
      },
      None => Self { head: None },
    }
  }

  pub fn head_with_tail(&self) -> Option<(Arc<Mutex<SystemMessageEntry>>, EarliestFirstSystemMessageList)> {
    self.head.as_ref().map(|head_arc| {
      let head_guard = head_arc.lock().unwrap();
      (
        head_arc.clone(),
        EarliestFirstSystemMessageList {
          head: head_guard.next.as_ref().cloned(),
        },
      )
    })
  }

  fn prepend_for_arc(self, msg_arc_opt: Option<Arc<Mutex<SystemMessageEntry>>>) -> EarliestFirstSystemMessageList {
    let msg_arc = msg_arc_opt.unwrap();
    let msg_arc_cloned = msg_arc.clone();
    let mut msg_inner = msg_arc.lock().unwrap();
    msg_inner.set_next(self.head);
    EarliestFirstSystemMessageList {
      head: Some(msg_arc_cloned),
    }
  }

  pub fn reverse_prepend(self, other: LatestFirstSystemMessageList) -> Self {
    let mut remaining = other;
    let mut result = self;
    while remaining.non_empty() {
      let msg = remaining.head.as_ref().cloned();
      remaining = remaining.tail();
      result = result.prepend_for_arc(msg)
    }
    result
  }
}

impl SystemMessageList for EarliestFirstSystemMessageList {
  type Other = LatestFirstSystemMessageList;

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

  fn tail(&self) -> EarliestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|e| {
      let inner = e.lock().unwrap();
      inner.next.clone()
    });
    EarliestFirstSystemMessageList { head: next }
  }

  fn prepend(self, msg: SystemMessageEntry) -> EarliestFirstSystemMessageList {
    self.prepend_for_arc(Some(Arc::new(Mutex::new(msg))))
  }

  fn reverse(self) -> Self::Other {
    LatestFirstSystemMessageList {
      head: reverse_inner(self.head, None),
    }
  }
}
