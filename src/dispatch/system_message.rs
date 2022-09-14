use std::sync::{Arc, Mutex};

use crate::actor::actor::ActorError;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::local_actor_ref_ref::LocalActorRefRef;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::message::Message;
use crate::ActuatorError;
use once_cell::sync::Lazy;
use std::error::Error;
use std::fmt::Debug;

pub trait SystemMessageQueueBehavior<Msg: Message>: Debug {
  fn system_enqueue(&mut self, receiver: ActorRefRef<Msg>, message: &mut SystemMessage);
  fn system_drain(&mut self, new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList;
  fn has_system_messages(&self) -> bool;
}

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

// fn size_inner(head: Option<SystemMessageArcMut>, acc: usize) -> usize {
//   head.map_or(acc, |e| size_inner(e.lock().unwrap().next(), acc + 1))
// }

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
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    failure: Option<ActuatorError>,
  },
  Recreate {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    cause: ActorError,
  },
  Suspend {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  },
  Resume {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    caused_by_failure: ActorError,
  },
  Terminate {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  },
  Supervise {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    child: ActorRefRef<AnyMessage>,
    a_sync: bool,
  },
  Watch {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  },
  NoMessage {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  },
  Failed {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    child: ActorRefRef<AnyMessage>,
    error: ActorError,
    uid: u32,
  },
  DeathWatchNotification {
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
    actor: ActorRefRef<AnyMessage>,
    existence_confirmed: bool,
    address_terminated: bool,
  },
}

unsafe impl Send for SystemMessage {}
unsafe impl Sync for SystemMessage {}

impl SystemMessage {
  pub fn of_create(failure: Option<ActuatorError>) -> Self {
    Self::of_create_with_next_opt(failure, None)
  }

  pub fn of_create_with_next_opt(failure: Option<ActuatorError>, next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Create { next_opt, failure }
  }

  pub fn of_recreate(cause: ActorError) -> Self {
    Self::of_recreate_with_next_opt(cause, None)
  }

  pub fn of_recreate_with_next_opt(cause: ActorError, next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Recreate { next_opt, cause }
  }

  pub fn of_suspend() -> Self {
    Self::of_suspend_with_next_opt(None)
  }

  pub fn of_suspend_with_next_opt(next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Suspend { next_opt }
  }

  pub fn of_resume(caused_by_failure: ActorError) -> Self {
    Self::of_resume_with_next_opt(caused_by_failure, None)
  }

  pub fn of_resume_with_next_opt(caused_by_failure: ActorError, next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Resume {
      next_opt,
      caused_by_failure,
    }
  }

  pub fn of_terminate() -> Self {
    Self::of_terminate_with_next_opt(None)
  }

  pub fn of_terminate_with_next_opt(next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Terminate { next_opt }
  }

  pub fn of_supervise(child: ActorRefRef<AnyMessage>, a_sync: bool) -> Self {
    Self::of_supervise_with_next_opt(child, a_sync, None)
  }

  pub fn of_supervise_with_next_opt(
    child: ActorRefRef<AnyMessage>,
    a_sync: bool,
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  ) -> Self {
    SystemMessage::Supervise {
      next_opt,
      child,
      a_sync,
    }
  }

  pub fn of_watch() -> Self {
    Self::of_watch_with_next_opt(None)
  }

  pub fn of_watch_with_next_opt(next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::Watch { next_opt }
  }

  pub fn of_no_message() -> Self {
    Self::of_no_message_with_next_opt(None)
  }

  pub fn of_no_message_with_next_opt(next_opt: Option<Arc<Mutex<SystemMessage>>>) -> Self {
    SystemMessage::NoMessage { next_opt }
  }

  pub fn unlink(&mut self) {
    self.set_next(None);
  }

  pub fn is_unlinked(&self) -> bool {
    self.next().is_none()
  }

  pub fn is_no_message(&self) -> bool {
    match self {
      SystemMessage::NoMessage { .. } => true,
      _ => false,
    }
  }

  pub fn of_failed(child: ActorRefRef<AnyMessage>, error: ActorError, uid: u32) -> Self {
    Self::of_failed_with_next_opt(child, error, uid, None)
  }

  pub fn of_failed_with_next_opt(
    child: ActorRefRef<AnyMessage>,
    error: ActorError,
    uid: u32,
    next_opt: Option<Arc<Mutex<SystemMessage>>>,
  ) -> Self {
    SystemMessage::Failed {
      child,
      error,
      uid,
      next_opt,
    }
  }

  pub fn next(&self) -> Option<Arc<Mutex<SystemMessage>>> {
    match self {
      SystemMessage::Create { next_opt, .. } => next_opt.clone(),
      SystemMessage::Recreate { next_opt, .. } => next_opt.clone(),
      SystemMessage::Suspend { next_opt } => next_opt.clone(),
      SystemMessage::Resume { next_opt, .. } => next_opt.clone(),
      SystemMessage::Terminate { next_opt } => next_opt.clone(),
      SystemMessage::Supervise { next_opt, .. } => next_opt.clone(),
      SystemMessage::Watch { next_opt } => next_opt.clone(),
      SystemMessage::NoMessage { next_opt } => next_opt.clone(),
      SystemMessage::Failed { next_opt, .. } => next_opt.clone(),
      SystemMessage::DeathWatchNotification { next_opt, .. } => next_opt.clone(),
    }
  }

  fn clone_arc(next: &Option<Arc<Mutex<SystemMessage>>>) -> Option<Arc<Mutex<SystemMessage>>> {
    next.as_ref().map(|e| Arc::clone(&e))
  }

  pub fn set_next(&mut self, value: Option<Arc<Mutex<SystemMessage>>>) {
    match self {
      SystemMessage::Create { failure, .. } => {
        *self = SystemMessage::Create {
          next_opt: value,
          failure: failure.clone(),
        }
      }
      SystemMessage::Recreate { cause, .. } => {
        *self = SystemMessage::Recreate {
          next_opt: value,
          cause: cause.clone(),
        }
      }
      SystemMessage::Suspend { .. } => *self = SystemMessage::Suspend { next_opt: value },
      SystemMessage::Resume { caused_by_failure, .. } => {
        *self = SystemMessage::Resume {
          next_opt: value,
          caused_by_failure: caused_by_failure.clone(),
        }
      }
      SystemMessage::Terminate { .. } => *self = SystemMessage::Terminate { next_opt: value },
      SystemMessage::Supervise { child, a_sync, .. } => {
        *self = SystemMessage::Supervise {
          next_opt: value,
          child: child.clone(),
          a_sync: *a_sync,
        }
      }
      SystemMessage::Watch { .. } => *self = SystemMessage::Watch { next_opt: value },
      SystemMessage::NoMessage { .. } => *self = SystemMessage::NoMessage { next_opt: value },
      SystemMessage::Failed { child, error, uid, .. } => {
        *self = SystemMessage::Failed {
          next_opt: value,
          child: child.clone(),
          error: error.clone(),
          uid: *uid,
        }
      }
      SystemMessage::DeathWatchNotification {
        actor,
        existence_confirmed,
        address_terminated,
        ..
      } => {
        *self = SystemMessage::DeathWatchNotification {
          next_opt: value,
          actor: actor.clone(),
          existence_confirmed: *existence_confirmed,
          address_terminated: *address_terminated,
        }
      }
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

pub const LNIL: Lazy<LatestFirstSystemMessageList> = Lazy::new(|| LatestFirstSystemMessageList::new(None));

pub const ENIL: Lazy<EarliestFirstSystemMessageList> = Lazy::new(|| EarliestFirstSystemMessageList::new(None));

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

  pub fn head_with_tail(&self) -> Option<(Arc<Mutex<SystemMessage>>, EarliestFirstSystemMessageList)> {
    self.head.as_ref().map(|head_arc| {
      let head_guard = head_arc.lock().unwrap();
      (
        head_arc.clone(),
        EarliestFirstSystemMessageList {
          head: head_guard.next(),
        },
      )
    })
  }

  fn prepend_for_arc(self, msg_arc_opt: Option<Arc<Mutex<SystemMessage>>>) -> EarliestFirstSystemMessageList {
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

  #[ctor::ctor]
  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    let _ = env_logger::try_init();
  }

  #[test]
  fn test_of_create() {
    let msg = SystemMessage::of_create(None);
    match msg {
      SystemMessage::Create {
        next_opt,
        failure: msg_failure,
      } => {
        assert!(next_opt.is_none());
        assert_eq!(msg_failure, None);
      }
      _ => panic!("Expected SystemMessage::Create variant"),
    }
  }

  #[test]
  fn test_of_recreate() {
    let cause = ActorError::ActorFailed {
      message: "test".to_string(),
    };
    let sys_msg = SystemMessage::of_recreate(cause.clone());
    match sys_msg {
      SystemMessage::Recreate {
        next_opt,
        cause: actual_cause,
      } => {
        assert!(next_opt.is_none());
        assert_eq!(actual_cause, cause);
      }
      _ => panic!("Unexpected message type"),
    }
  }

  #[test]
  fn test() {
    let earliest_first_system_message_list_1 =
      EarliestFirstSystemMessageList::new(Some(SystemMessage::of_create(None)));
    assert_eq!(earliest_first_system_message_list_1.size(), 1);

    let system_message = SystemMessage::of_suspend();
    let earliest_first_system_message_list_2 = earliest_first_system_message_list_1.prepend(system_message);
    assert_eq!(earliest_first_system_message_list_2.size(), 2);

    let cause = ActorError::ActorFailed {
      message: "test".to_string(),
    };
    let system_message = SystemMessage::of_resume(cause);
    let earliest_first_system_message_list_3 = earliest_first_system_message_list_2.prepend(system_message);
    assert_eq!(earliest_first_system_message_list_3.size(), 3);

    let latest_first_system_message_list_3 = earliest_first_system_message_list_3.clone().reverse();
    assert_eq!(latest_first_system_message_list_3.size(), 3);

    let earliest_first_system_message_list_4 = latest_first_system_message_list_3.reverse();
    assert_eq!(earliest_first_system_message_list_4.size(), 3);

    assert_eq!(
      earliest_first_system_message_list_3,
      earliest_first_system_message_list_4
    );
  }
}
