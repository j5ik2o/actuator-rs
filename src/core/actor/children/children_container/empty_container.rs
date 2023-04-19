use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::children::child_state::ChildState;
use crate::core::actor::children::children_container::normal_container::NormalContainer;
use crate::core::actor::children::children_container::{ChildrenContainer, ChildrenContainerBehavior};
use crate::core::dispatch::any_message::AnyMessage;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct EmptyContainerInner {
  empty_stats: BTreeMap<String, ChildState>,
}

impl PartialEq for EmptyContainerInner {
  fn eq(&self, other: &Self) -> bool {
    self.empty_stats == other.empty_stats
  }
}

#[derive(Debug, Clone)]
pub struct EmptyContainer {
  inner: Arc<Mutex<EmptyContainerInner>>,
}

impl PartialEq for EmptyContainer {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    &*l == &*r
  }
}

impl EmptyContainer {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(EmptyContainerInner {
        empty_stats: BTreeMap::new(),
      })),
    }
  }

  pub fn deep_copy(&self) -> Self {
    Self {
      inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
    }
  }
}

impl ChildrenContainerBehavior for EmptyContainer {
  fn add(&mut self, name: &str, stats: ChildState) -> ChildrenContainer {
    let mut inner = self.inner.lock().unwrap();
    inner.empty_stats.insert(name.to_string(), stats);
    ChildrenContainer::Normal(NormalContainer::new_with_children(inner.empty_stats.clone()))
  }

  fn remove(&mut self, _child: ActorRef<AnyMessage>) -> ChildrenContainer {
    ChildrenContainer::Empty(self.clone())
  }

  fn get_all_child_stats(&self) -> Vec<ChildState> {
    vec![]
  }

  fn get_by_name(&self, _name: &str) -> Option<ChildState> {
    None
  }

  fn get_by_ref(&self, _actor: ActorRef<AnyMessage>) -> Option<ChildState> {
    None
  }

  fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    log::debug!("@@@ Children: []");
    vec![]
  }

  fn stats(&self) -> Vec<ChildState> {
    vec![]
  }

  fn stats_mut<F>(&mut self, _f: F)
  where
    F: Fn(&mut ChildState), {
  }

  fn shall_die(&mut self, _actor: ActorRef<AnyMessage>) -> ChildrenContainer {
    ChildrenContainer::Empty(self.clone())
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer {
    let mut inner = self.inner.lock().unwrap();
    inner
      .empty_stats
      .insert(name.to_string(), ChildState::ChildNameReserved);
    ChildrenContainer::Normal(NormalContainer::new_with_children(inner.empty_stats.clone()))
  }

  fn un_reserve(&mut self, _name: &str) -> ChildrenContainer {
    ChildrenContainer::Empty(self.clone())
  }
}
