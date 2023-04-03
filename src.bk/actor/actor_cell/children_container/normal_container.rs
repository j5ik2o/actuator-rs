use crate::actor::actor_cell::children_container::terminating_container::TerminatingContainerRef;
use crate::actor::actor_cell::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::child_state::ChildState;
use crate::dispatch::message::Message;
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct NormalContainer<Msg: Message> {
  children: BTreeMap<String, ChildState<Msg>>,
  reserved_names: HashSet<String>,
}

impl<Msg: Message> NormalContainer<Msg> {
  pub fn new() -> Self {
    Self {
      children: BTreeMap::new(),
      reserved_names: HashSet::new(),
    }
  }

  pub fn new_with_children(children: BTreeMap<String, ChildState<Msg>>) -> Self {
    Self {
      children,
      reserved_names: HashSet::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct NormalContainerRef<Msg: Message> {
  inner: Arc<Mutex<NormalContainer<Msg>>>,
}

impl<Msg: Message> PartialEq for NormalContainerRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    l.children == r.children
  }
}

impl<Msg: Message> NormalContainerRef<Msg> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(NormalContainer::new())),
    }
  }

  pub fn new_with_children(children: BTreeMap<String, ChildState<Msg>>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(NormalContainer::new_with_children(children))),
    }
  }

  pub fn deep_copy(&self) -> NormalContainerRef<Msg> {
    Self {
      inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
    }
  }
}

impl<Msg: Message> ChildrenContainerBehavior<Msg> for NormalContainerRef<Msg> {
  fn add(&mut self, name: &str, stats: ChildState<Msg>) -> ChildrenContainer<Msg> {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      panic!("Name is reserved");
    } else {
      lock.children.insert(name.to_string(), stats);
      ChildrenContainer::Normal(self.clone())
    }
  }

  fn remove(&mut self, child: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    let mut lock = self.inner.lock().unwrap();
    let mut to_remove = Vec::new();
    for (name, state) in lock.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if stats.child == child {
          to_remove.push(name.clone());
        }
      }
    }
    for name in to_remove {
      lock.children.remove(&name);
    }
    ChildrenContainer::Normal(self.clone())
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState<Msg>> {
    let lock = self.inner.lock().unwrap();
    lock.children.get(name).cloned()
  }

  fn get_by_ref(&self, actor: ActorRefRef<Msg>) -> Option<ChildState<Msg>> {
    let lock = self.inner.lock().unwrap();
    for (_, state) in lock.children.iter() {
      if let ChildState::ChildRestartStats(stats) = state {
        if stats.child == actor {
          return Some(state.clone());
        }
      }
    }
    None
  }

  fn children(&self) -> Vec<ActorRefRef<Msg>> {
    let lock = self.inner.lock().unwrap();
    lock
      .children
      .values()
      .filter_map(|state| match state {
        ChildState::ChildRestartStats(stats) => Some(stats.child.clone()),
        _ => None,
      })
      .collect()
  }

  fn stats(&self) -> Vec<ChildState<Msg>> {
    let lock = self.inner.lock().unwrap();
    lock.children.values().cloned().collect()
  }

  fn shall_die(&mut self, actor: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    let lock = self.inner.lock().unwrap();
    ChildrenContainer::Terminating(TerminatingContainerRef::new(
      lock.children.clone(),
      vec![actor],
      SuspendReason::UserRequest,
    ))
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      panic!("Name already reserved");
    } else {
      lock.reserved_names.insert(name.to_string());
      ChildrenContainer::Normal(self.clone())
    }
  }

  fn unreserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    let mut lock = self.inner.lock().unwrap();
    if lock.reserved_names.contains(name) {
      lock.reserved_names.remove(name);
      ChildrenContainer::Normal(self.clone())
    } else {
      panic!("Name is not reserved");
    }
  }
}
