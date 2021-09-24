use std::alloc::System;
use std::sync::{Arc, Mutex};
use crate::kernel::system_message::SystemMessage::{Create, Recreate};
use std::ops::Deref;
use crate::kernel::ActorRef;

#[derive(Clone)]
pub enum SystemMessage {
  Create {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Recreate {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Suspend {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Resume {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Terminate {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Supervise {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  Watch {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
  NoMessage {
    next: Option<Arc<Mutex<SystemMessage>>>,
  },
    Failed {
        child: Arc<dyn ActorRef>
    }
}

impl SystemMessage {
  fn next(&self) -> Option<Arc<Mutex<SystemMessage>>> {
    match self {
      Create { next } => next.as_ref().map(|e| Arc::clone(&e)),
      Recreate { next } => next.as_ref().map(|e| Arc::clone(&e)),
      _ => None,
    }
  }

  fn set_next(&mut self, value: Option<Arc<Mutex<SystemMessage>>>) {
    match self {
      Create { .. } => *self = Create { next: value },
      _ => {}
    }
  }
}

pub struct LatestFirstSystemMessageList {
  head: Option<Arc<Mutex<SystemMessage>>>,
}

fn size_inner(head: Option<Arc<Mutex<SystemMessage>>>, acc: u32) -> u32 {
  if head.is_none() {
    acc
  } else {
    size_inner(
      head.and_then(|e| {
        let inner = e.lock().unwrap();
        inner.next()
      }),
      acc + 1,
    )
  }
}

fn reverse_inner(
  head: Option<Arc<Mutex<SystemMessage>>>,
  acc: Option<Arc<Mutex<SystemMessage>>>,
) -> Option<Arc<Mutex<SystemMessage>>> {
  match head {
    None => acc,
    Some(head_arc) => {
      let head_arc_cloned = head_arc.clone();
      let mut head_inner = head_arc.lock().unwrap();
      let next = head_inner.next();
      head_inner.set_next(acc);
      reverse_inner(next, Some(head_arc_cloned))
    }
  }
}

impl LatestFirstSystemMessageList {
  fn is_empty(&self) -> bool {
    self.head.is_none()
  }
  fn non_empty(&self) -> bool {
    !self.is_empty()
  }

  fn size(&self) -> u32 {
    size_inner(
      self.head.as_ref().and_then(|e| {
        let inner = e.lock().unwrap();
        inner.next()
      }),
      0,
    )
  }

  fn tail(&self) -> LatestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|e| {
      let inner = e.lock().unwrap();
      inner.next()
    });
    LatestFirstSystemMessageList { head: next }
  }

  fn prepend(self, mut msg: SystemMessage) -> LatestFirstSystemMessageList {
    msg.set_next(self.head);
    LatestFirstSystemMessageList {
      head: Some(Arc::new(Mutex::new(msg))),
    }
  }
}

pub struct EarliestFirstSystemMessageList {
  head: Option<Arc<Mutex<SystemMessage>>>,
}

impl EarliestFirstSystemMessageList {
  fn is_empty(&self) -> bool {
    self.head.is_none()
  }
  fn non_empty(&self) -> bool {
    !self.is_empty()
  }
  fn size(&self) -> u32 {
    size_inner(
      self.head.as_ref().and_then(|e| {
        let inner = e.lock().unwrap();
        inner.next()
      }),
      0,
    )
  }
  fn tail(&self) -> EarliestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|e| {
      let inner = e.lock().unwrap();
      inner.next()
    });
    EarliestFirstSystemMessageList { head: next }
  }

  fn reverse(self) -> EarliestFirstSystemMessageList {
    EarliestFirstSystemMessageList {
      head: reverse_inner(self.head, None),
    }
  }

  fn prepend(self, msg_opt: Option<Arc<Mutex<SystemMessage>>>) -> EarliestFirstSystemMessageList {
    let msg = msg_opt.unwrap();
    let msg_cloned = msg.clone();
    let mut msg_inner = msg.lock().unwrap();
    msg_inner.set_next(self.head);
    EarliestFirstSystemMessageList {
      head: Some(msg_cloned),
    }
  }

  fn reverse_prepend(self, other: LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    let mut remaining = other;
    let mut result = self;
    while remaining.non_empty() {
      let mut msg = remaining.head.clone();
      remaining = remaining.tail();
      result = result.prepend(msg)
    }
    result
  }
}
