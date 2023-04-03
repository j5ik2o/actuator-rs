use crate::actor::actor_cell::children_container::normal_container::NormalContainerRef;
use crate::actor::actor_cell::children_container::{ChildrenContainer, ChildrenContainerBehavior};
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::child_state::ChildState;
use crate::dispatch::message::Message;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct EmptyContainer<Msg: Message> {
  empty_stats: BTreeMap<String, ChildState<Msg>>,
}

impl<Msg: Message> PartialEq for EmptyContainer<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.empty_stats == other.empty_stats
  }
}

#[derive(Debug, Clone)]
pub struct EmptyContainerRef<Msg: Message> {
  inner: Arc<Mutex<EmptyContainer<Msg>>>,
}

impl<Msg: Message> PartialEq for EmptyContainerRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    let l = self.inner.lock().unwrap();
    let r = other.inner.lock().unwrap();
    &*l == &*r
  }
}

impl<Msg: Message> EmptyContainerRef<Msg> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(EmptyContainer {
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

impl<Msg: Message> ChildrenContainerBehavior<Msg> for EmptyContainerRef<Msg> {
  fn add(&mut self, name: &str, stats: ChildState<Msg>) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    inner.empty_stats.insert(name.to_string(), stats);
    ChildrenContainer::Normal(NormalContainerRef::new_with_children(inner.empty_stats.clone()))
  }

  fn remove(&mut self, _child: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    ChildrenContainer::Empty(self.clone())
  }

  fn get_by_name(&self, _name: &str) -> Option<ChildState<Msg>> {
    None
  }

  fn get_by_ref(&self, _actor: ActorRefRef<Msg>) -> Option<ChildState<Msg>> {
    None
  }

  fn children(&self) -> Vec<ActorRefRef<Msg>> {
    vec![]
  }

  fn stats(&self) -> Vec<ChildState<Msg>> {
    vec![]
  }

  fn shall_die(&mut self, _actor: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    ChildrenContainer::Empty(self.clone())
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    let mut inner = self.inner.lock().unwrap();
    inner
      .empty_stats
      .insert(name.to_string(), ChildState::ChildNameReserved);
    ChildrenContainer::Normal(NormalContainerRef::new_with_children(inner.empty_stats.clone()))
  }

  fn unreserve(&mut self, _name: &str) -> ChildrenContainer<Msg> {
    ChildrenContainer::Empty(self.clone())
  }
}
