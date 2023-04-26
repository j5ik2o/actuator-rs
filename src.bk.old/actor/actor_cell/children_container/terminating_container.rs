use crate::actor::actor_cell::children_container::normal_container::NormalContainerRef;
use crate::actor::actor_cell::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::actor::actor_path::ActorPathBehavior;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::ActorRefBehavior;
use crate::actor::child_state::ChildState;
use crate::dispatch::message::Message;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct TerminatingContainer<Msg: Message> {
  children: BTreeMap<String, ChildState<Msg>>,
  to_die: Vec<ActorRefRef<Msg>>,
  suspend_reason: SuspendReason,
}

impl<Msg: Message> PartialEq for TerminatingContainer<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.children == other.children && self.to_die == other.to_die && self.suspend_reason == other.suspend_reason
  }
}

impl<Msg: Message> TerminatingContainer<Msg> {
  pub fn new(
    children: BTreeMap<String, ChildState<Msg>>,
    to_die: Vec<ActorRefRef<Msg>>,
    suspend_reason: SuspendReason,
  ) -> Self {
    Self {
      children,
      to_die,
      suspend_reason,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TerminatingContainerRef<Msg: Message> {
  inner: Arc<Mutex<TerminatingContainer<Msg>>>,
}

impl<Msg: Message> PartialEq for TerminatingContainerRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    &*l == &*r
  }
}

impl<Msg: Message> TerminatingContainerRef<Msg> {
  pub fn new(
    children: BTreeMap<String, ChildState<Msg>>,
    to_die: Vec<ActorRefRef<Msg>>,
    suspend_reason: SuspendReason,
  ) -> Self {
    Self {
      inner: Arc::new(Mutex::new(TerminatingContainer::new(children, to_die, suspend_reason))),
    }
  }

  pub fn deep_copy(&self) -> TerminatingContainerRef<Msg> {
    Self {
      inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
    }
  }

  pub fn set_reason(&mut self, reason: SuspendReason) {
    let mut inner = self.inner.lock().unwrap();
    inner.suspend_reason = reason;
  }
}

impl<Msg: Message> ChildrenContainerBehavior<Msg> for TerminatingContainerRef<Msg> {
  fn add(&mut self, name: &str, stats: ChildState<Msg>) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    inner.children.insert(name.to_string(), stats);
    ChildrenContainer::Terminating(self.clone())
  }

  fn remove(&mut self, child: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    inner.to_die.retain(|c| c.ne(&child));
    if inner.to_die.is_empty() {
      match inner.suspend_reason {
        SuspendReason::Termination => ChildrenContainer::Terminated,
        _ => {
          inner
            .children
            .retain(|k, _| k.to_string() != child.path().name().to_string());
          ChildrenContainer::Normal(NormalContainerRef::new_with_children(inner.children.clone()))
        }
      }
    } else {
      inner
        .children
        .retain(|k, _| k.to_string() != child.path().name().to_string());
      ChildrenContainer::Terminating(self.clone())
    }
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState<Msg>> {
    let inner = self.inner.lock().unwrap();
    inner.children.get(name).cloned()
  }

  fn get_by_ref(&self, actor: ActorRefRef<Msg>) -> Option<ChildState<Msg>> {
    let inner = self.inner.lock().unwrap();
    for (_, state) in inner.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if stats.child == actor {
          return Some(state.clone());
        }
      }
    }
    None
  }

  fn children(&self) -> Vec<ActorRefRef<Msg>> {
    let inner = self.inner.lock().unwrap();
    inner
      .children
      .values()
      .filter_map(|state| match state {
        ChildState::ChildRestartStats(stats) => Some(stats.child.clone()),
        _ => None,
      })
      .collect()
  }

  fn stats(&self) -> Vec<ChildState<Msg>> {
    let inner = self.inner.lock().unwrap();
    inner.children.values().cloned().collect()
  }

  fn shall_die(&mut self, actor: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    inner.to_die.push(actor);
    ChildrenContainer::Terminating(self.clone())
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    match inner.suspend_reason {
      SuspendReason::Termination => panic!("cannot reserve actor name '{}': terminating", name),
      _ => {
        if inner.children.contains_key(name) {
          panic!("actor name [{}] is not unique!", name);
        } else {
          inner.children.insert(name.to_string(), ChildState::ChildNameReserved);
          ChildrenContainer::Terminating(self.clone())
        }
      }
    }
  }

  fn unreserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    match inner.children.get(name) {
      Some(ChildState::ChildNameReserved) => {
        inner.children.remove(name);
        ChildrenContainer::Terminating(self.clone())
      }
      _ => ChildrenContainer::Terminating(self.clone()),
    }
  }

  fn is_terminating(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.suspend_reason == SuspendReason::Termination
  }

  fn is_normal(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.suspend_reason == SuspendReason::UserRequest
  }
}
