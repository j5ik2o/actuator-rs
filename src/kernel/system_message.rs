use std::sync::{Arc, Mutex};
use crate::kernel::system_message::SystemMessage::*;

use crate::kernel::ActorRef;
use std::error::Error;
use once_cell::sync::Lazy;

fn size_inner(head: Option<Arc<Mutex<SystemMessage>>>, acc: usize) -> usize {
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
      let next = {
        let mut head_inner = head_arc.lock().unwrap();
        let next = head_inner.next();
        head_inner.set_next(acc);
        next
      };
      reverse_inner(next, Some(head_arc_cloned))
    }
  }
}

#[derive(Debug, Clone)]
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
    next: Option<Arc<Mutex<SystemMessage>>>,
    child: Arc<dyn ActorRef>,
    error: Arc<dyn Error + Sync + Send>,
    uid: u32,
  },
  DeathWatchNotification {
    next: Option<Arc<Mutex<SystemMessage>>>,
    actor: Arc<dyn ActorRef>,
    existence_confirmed: bool,
    address_terminated: bool,
  },
}

unsafe impl Sync for SystemMessage {}

impl SystemMessage {
  pub fn of_create(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Create {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Create { next: None },
    }
  }

  pub fn of_recreate(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Recreate {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Recreate { next: None },
    }
  }

  pub fn of_suspend(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Suspend {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Suspend { next: None },
    }
  }

  pub fn of_resume(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Resume {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Resume { next: None },
    }
  }

  pub fn of_terminate(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Terminate {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Terminate { next: None },
    }
  }

  pub fn of_supervise(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Supervise {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Supervise { next: None },
    }
  }

  pub fn of_watch(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => Watch {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => Watch { next: None },
    }
  }

  pub fn of_no_message(next_opt: Option<SystemMessage>) -> Self {
    match next_opt {
      Some(next) => NoMessage {
        next: Some(Arc::new(Mutex::new(next))),
      },
      None => NoMessage { next: None },
    }
  }

  pub fn unlink(&mut self) {
    self.set_next(None);
  }

  pub fn unlinked(&self) -> bool {
    self.next().is_none()
  }

  pub fn is_no_message(&self) -> bool {
    match self {
      NoMessage { .. } => true,
      _ => false,
    }
  }

  pub fn next(&self) -> Option<Arc<Mutex<SystemMessage>>> {
    match self {
      Create { next } => Self::clone_arc(next),
      Recreate { next } => Self::clone_arc(next),
      Suspend { next } => Self::clone_arc(next),
      Resume { next } => Self::clone_arc(next),
      Terminate { next } => Self::clone_arc(next),
      Supervise { next } => Self::clone_arc(next),
      Watch { next } => Self::clone_arc(next),
      NoMessage { next } => Self::clone_arc(next),
      Failed { next, .. } => Self::clone_arc(next),
      DeathWatchNotification { next, .. } => Self::clone_arc(next),
      _ => None,
    }
  }

  fn clone_arc(next: &Option<Arc<Mutex<SystemMessage>>>) -> Option<Arc<Mutex<SystemMessage>>> {
    next.as_ref().map(|e| Arc::clone(&e))
  }

  pub fn set_next(&mut self, value: Option<Arc<Mutex<SystemMessage>>>) {
    match self {
      Create { .. } => *self = Create { next: value },
      Recreate { .. } => *self = Recreate { next: value },
      Suspend { .. } => *self = Suspend { next: value },
      Resume { .. } => *self = Resume { next: value },
      Terminate { .. } => *self = Terminate { next: value },
      Supervise { .. } => *self = Supervise { next: value },
      Watch { .. } => *self = Watch { next: value },
      NoMessage { .. } => *self = NoMessage { next: value },
      Failed {
        child, error, uid, ..
      } => {
        *self = Failed {
          next: value,
          child: Arc::clone(child),
          error: Arc::clone(error),
          uid: *uid,
        }
      }
      DeathWatchNotification {
        actor,
        existence_confirmed,
        address_terminated,
        ..
      } => {
        *self = DeathWatchNotification {
          next: value,
          actor: Arc::clone(actor),
          existence_confirmed: *existence_confirmed,
          address_terminated: *address_terminated,
        }
      }
      _ => {}
    }
  }
}

impl PartialEq for SystemMessage {
  fn eq(&self, other: &Self) -> bool {
    match (self.next().as_ref(), other.next().as_ref()) {
      (Some(v1_arc), Some(v2_arc)) => {
        if (v1_arc.as_ref() as *const _) == (v2_arc.as_ref() as *const _) {
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

pub trait SystemMessageList {
  type Other: SystemMessageList;

  fn is_empty(&self) -> bool;
  fn non_empty(&self) -> bool {
    !self.is_empty()
  }
  fn size(&self) -> usize;
  fn head(&self) -> Option<&Arc<Mutex<SystemMessage>>>;
  fn set_head(&mut self, value: Option<Arc<Mutex<SystemMessage>>>);
  fn tail(&self) -> Self;
  fn prepend(self, msg: SystemMessage) -> Self;

  fn reverse(self) -> Self::Other;
}

#[derive(Debug, Clone)]
pub struct LatestFirstSystemMessageList {
  head: Option<Arc<Mutex<SystemMessage>>>,
}

unsafe impl Sync for LatestFirstSystemMessageList {}

impl PartialEq for LatestFirstSystemMessageList {
  fn eq(&self, other: &Self) -> bool {
    match (&self.head, &other.head) {
      (Some(v1_arc), Some(v2_arc)) => {
        if (v1_arc.as_ref() as *const _) == (v2_arc.as_ref() as *const _) {
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

pub const LNIL: Lazy<LatestFirstSystemMessageList> =
  Lazy::new(|| LatestFirstSystemMessageList::new(None));
pub const ENIL: Lazy<EarliestFirstSystemMessageList> =
  Lazy::new(|| EarliestFirstSystemMessageList::new(None));

impl LatestFirstSystemMessageList {
  pub fn new(head_opt: Option<SystemMessage>) -> Self {
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

  fn head(&self) -> Option<&Arc<Mutex<SystemMessage>>> {
    self.head.as_ref()
  }

  fn set_head(&mut self, value: Option<Arc<Mutex<SystemMessage>>>) {
    self.head = value;
  }

  fn tail(&self) -> LatestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|system_message_arc| {
      let system_message_guard = system_message_arc.lock().unwrap();
      system_message_guard.next()
    });
    LatestFirstSystemMessageList { head: next }
  }

  fn prepend(self, mut msg: SystemMessage) -> LatestFirstSystemMessageList {
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

#[derive(Debug, Clone)]
pub struct EarliestFirstSystemMessageList {
  head: Option<Arc<Mutex<SystemMessage>>>,
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
  pub fn new(head_opt: Option<SystemMessage>) -> Self {
    match head_opt {
      Some(head) => Self {
        head: Some(Arc::new(Mutex::new(head))),
      },
      None => Self { head: None },
    }
  }

  fn prepend_for_arc(
    self,
    msg_arc_opt: Option<Arc<Mutex<SystemMessage>>>,
  ) -> EarliestFirstSystemMessageList {
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
      let msg = remaining.head.clone();
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

  fn head(&self) -> Option<&Arc<Mutex<SystemMessage>>> {
    self.head.as_ref()
  }

  fn set_head(&mut self, value: Option<Arc<Mutex<SystemMessage>>>) {
    self.head = value;
  }

  fn tail(&self) -> EarliestFirstSystemMessageList {
    let next = self.head.as_ref().and_then(|e| {
      let inner = e.lock().unwrap();
      inner.next()
    });
    EarliestFirstSystemMessageList { head: next }
  }

  fn prepend(self, msg: SystemMessage) -> EarliestFirstSystemMessageList {
    self.prepend_for_arc(Some(Arc::new(Mutex::new(msg))))
  }

  fn reverse(self) -> Self::Other {
    LatestFirstSystemMessageList {
      head: reverse_inner(self.head, None),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::env;

  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    // env::set_var("RUST_LOG", "trace");
    let _ = logger::try_init();
  }
  #[test]
  fn test() {
    init_logger();

    let e1 = EarliestFirstSystemMessageList::new(Some(SystemMessage::of_create(None)));

    println!("{:?}", e1);
    let size = e1.size();
    println!("l1.size = {}", size);

    let new_sm = SystemMessage::of_suspend(None);
    let e2 = e1.prepend(new_sm);

    println!("{:?}", e2);
    let size = e2.size();
    println!("l2.size = {}", size);

    let new_sm = SystemMessage::of_resume(None);
    let e3 = e2.prepend(new_sm);

    println!("{:?}", e3);
    let size = e3.size();
    println!("l3.size = {}", size);

    let l1 = e3.clone().reverse();
    println!("{:?}", l1);

    let e4 = l1.reverse();
    println!("{:?}", e4);

    assert_eq!(e3, e4);
  }
}
