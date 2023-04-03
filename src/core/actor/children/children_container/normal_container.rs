use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::children::child_state::ChildState;
use crate::core::actor::children::children_container::terminating_container::TerminatingContainer;
use crate::core::actor::children::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::core::dispatch::any_message::AnyMessage;

use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct NormalContainerInner {
  children: BTreeMap<String, ChildState>,
  reserved_names: HashSet<String>,
}

impl NormalContainerInner {
  pub fn new() -> Self {
    Self {
      children: BTreeMap::new(),
      reserved_names: HashSet::new(),
    }
  }

  pub fn new_with_children(children: BTreeMap<String, ChildState>) -> Self {
    Self {
      children,
      reserved_names: HashSet::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct NormalContainer {
  inner: Arc<Mutex<NormalContainerInner>>,
}

impl PartialEq for NormalContainer {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    l.children == r.children
  }
}

impl NormalContainer {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(NormalContainerInner::new())),
    }
  }

  pub fn new_with_children(children: BTreeMap<String, ChildState>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(NormalContainerInner::new_with_children(children))),
    }
  }

  pub fn deep_copy(&self) -> NormalContainer {
    Self {
      inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
    }
  }
}

impl ChildrenContainerBehavior for NormalContainer {
  fn add(&mut self, name: &str, stats: ChildState) -> ChildrenContainer {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      panic!("Name is reserved");
    } else {
      lock.children.insert(name.to_string(), stats);
      ChildrenContainer::Normal(self.clone())
    }
  }

  fn remove(&mut self, child: ActorRef<AnyMessage>) -> ChildrenContainer {
    let mut lock = self.inner.lock().unwrap();
    let mut to_remove = Vec::new();
    for (name, state) in lock.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if *stats.child_ref() == child {
          to_remove.push(name.clone());
        }
      }
    }
    for name in to_remove {
      lock.children.remove(&name);
    }
    ChildrenContainer::Normal(self.clone())
  }

  fn get_all_child_stats(&self) -> Vec<ChildState> {
    let lock = self.inner.lock().unwrap();
    lock.children.values().cloned().collect()
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState> {
    let lock = self.inner.lock().unwrap();
    lock.children.get(name).cloned()
  }

  fn get_by_ref(&self, actor: ActorRef<AnyMessage>) -> Option<ChildState> {
    let lock = self.inner.lock().unwrap();
    for (_, state) in lock.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if *stats.child_ref() == actor {
          return Some(state.clone());
        }
      }
    }
    None
  }

  fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    let lock = self.inner.lock().unwrap();
    lock
      .children
      .values()
      .filter_map(|state| match state {
        ChildState::ChildRestartStats(stats) => Some(stats.child_ref().clone()),
        _ => None,
      })
      .collect()
  }

  fn stats(&self) -> Vec<ChildState> {
    let lock = self.inner.lock().unwrap();
    lock.children.values().cloned().collect()
  }

  fn stats_mut<F>(&mut self, f: F)
  where
    F: Fn(&mut ChildState), {
    let mut lock = self.inner.lock().unwrap();
    lock.children.values_mut().for_each(f);
  }

  fn shall_die(&mut self, actor: ActorRef<AnyMessage>) -> ChildrenContainer {
    let lock = self.inner.lock().unwrap();
    ChildrenContainer::Terminating(TerminatingContainer::new(
      lock.children.clone(),
      vec![actor],
      SuspendReason::UserRequest,
    ))
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      panic!("Name already reserved");
    } else {
      lock.reserved_names.insert(name.to_string());
      ChildrenContainer::Normal(self.clone())
    }
  }

  fn un_reserve(&mut self, name: &str) -> ChildrenContainer {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      lock.reserved_names.remove(name);
      ChildrenContainer::Normal(self.clone())
    } else {
      panic!("Name is not reserved");
    }
  }
}
