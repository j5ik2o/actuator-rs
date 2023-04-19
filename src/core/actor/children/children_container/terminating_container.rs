use crate::core::actor::actor_path::ActorPathBehavior;
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::actor::children::child_state::ChildState;
use crate::core::actor::children::children_container::normal_container::NormalContainer;
use crate::core::actor::children::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::core::dispatch::any_message::AnyMessage;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct TerminatingContainerInner {
  children: BTreeMap<String, ChildState>,
  to_die: Vec<ActorRef<AnyMessage>>,
  suspend_reason: SuspendReason,
}

impl PartialEq for TerminatingContainerInner {
  fn eq(&self, other: &Self) -> bool {
    self.children == other.children && self.to_die == other.to_die && self.suspend_reason == other.suspend_reason
  }
}

impl TerminatingContainerInner {
  pub fn new(
    children: BTreeMap<String, ChildState>,
    to_die: Vec<ActorRef<AnyMessage>>,
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
pub struct TerminatingContainer {
  inner: Arc<Mutex<TerminatingContainerInner>>,
}

impl PartialEq for TerminatingContainer {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    &*l == &*r
  }
}

impl TerminatingContainer {
  pub fn new(
    children: BTreeMap<String, ChildState>,
    to_die: Vec<ActorRef<AnyMessage>>,
    suspend_reason: SuspendReason,
  ) -> Self {
    Self {
      inner: Arc::new(Mutex::new(TerminatingContainerInner::new(
        children,
        to_die,
        suspend_reason,
      ))),
    }
  }

  pub fn deep_copy(&self) -> TerminatingContainer {
    Self {
      inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
    }
  }

  pub fn get_children(&self) -> BTreeMap<String, ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.clone()
  }

  pub fn to_die(&self) -> Vec<ActorRef<AnyMessage>> {
    let inner = self.inner.lock().unwrap();
    inner.to_die.clone()
  }

  pub fn set_reason(&mut self, reason: SuspendReason) {
    let mut inner = self.inner.lock().unwrap();
    inner.suspend_reason = reason;
  }

  pub fn get_reason(&self) -> SuspendReason {
    let inner = self.inner.lock().unwrap();
    inner.suspend_reason.clone()
  }
}

impl ChildrenContainerBehavior for TerminatingContainer {
  fn add(&mut self, name: &str, stats: ChildState) -> ChildrenContainer {
    let mut inner = self.inner.lock().unwrap();
    inner.children.insert(name.to_string(), stats);
    ChildrenContainer::Terminating(self.clone())
  }

  fn remove(&mut self, child: ActorRef<AnyMessage>) -> ChildrenContainer {
    let mut inner = self.inner.lock().unwrap();
    inner.to_die.retain(|c| c.ne(&child));
    if inner.to_die.is_empty() {
      match inner.suspend_reason {
        SuspendReason::Termination => ChildrenContainer::Terminated,
        _ => {
          inner
            .children
            .retain(|k, _| k.to_string() != child.path().name().to_string());
          ChildrenContainer::Normal(NormalContainer::new_with_children(inner.children.clone()))
        }
      }
    } else {
      inner
        .children
        .retain(|k, _| k.to_string() != child.path().name().to_string());
      ChildrenContainer::Terminating(self.clone())
    }
  }

  fn get_all_child_stats(&self) -> Vec<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.values().cloned().collect()
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.get(name).cloned()
  }

  fn get_by_ref(&self, actor: ActorRef<AnyMessage>) -> Option<ChildState> {
    let inner = self.inner.lock().unwrap();
    for (_, state) in inner.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if *stats.child_ref() == actor {
          return Some(state.clone());
        }
      }
    }
    None
  }

  fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    let inner = self.inner.lock().unwrap();
    log::debug!("@@@ Children: {:?}", inner.children);
    let result = inner
      .children
      .values()
      .filter_map(|state| match state {
        ChildState::ChildRestartStats(stats) => Some(stats.child_ref().clone()),
        _ => None,
      })
      .collect();
    log::debug!("@@@ Children: {:?}", result);
    result
  }

  fn stats(&self) -> Vec<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.values().cloned().collect()
  }

  fn stats_mut<F>(&mut self, f: F)
  where
    F: Fn(&mut ChildState), {
    let mut inner = self.inner.lock().unwrap();
    inner.children.values_mut().for_each(f)
  }

  fn shall_die(&mut self, actor: ActorRef<AnyMessage>) -> ChildrenContainer {
    let mut inner = self.inner.lock().unwrap();
    inner.to_die.push(actor);
    ChildrenContainer::Terminating(self.clone())
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer {
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

  fn un_reserve(&mut self, name: &str) -> ChildrenContainer {
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
