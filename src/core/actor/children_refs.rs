use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base64_string_rs::Base64StringFactory;
use rand::RngCore;

use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::ActorPath;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::child_state::{ChildRestartStats, ChildState};

use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;

#[derive(Debug, Clone)]
struct ChildrenRefsInner {
  children: BTreeMap<String, ChildState>,
  reserved_names: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ChildrenRefs {
  inner: Arc<Mutex<ChildrenRefsInner>>,
}

impl ChildrenRefs {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(ChildrenRefsInner {
        children: BTreeMap::new(),
        reserved_names: HashSet::new(),
      })),
    }
  }

  pub fn clear(&mut self) {
    *self = Self::new();
  }

  pub fn is_empty(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.children.is_empty()
  }

  pub fn non_empty(&self) -> bool {
    !self.is_empty()
  }

  pub fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    let inner = self.inner.lock().unwrap();
    let result = inner
      .children
      .values()
      .filter_map(|state| match state {
        ChildState::ChildRestartStats(stats) => Some(stats.child_ref().clone()),
        _ => None,
      })
      .collect();
    result
  }

  pub fn stop_all_children(&self) {
    let mut inner = self.inner.lock().unwrap();
    for state in inner.children.values_mut() {
      match state {
        ChildState::ChildRestartStats(stats) => {
          let actor_ref = stats.child_ref_mut();
          actor_ref.stop();
        }
        _ => {}
      }
    }
  }

  pub fn get_all_child_stats(&self) -> Vec<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.values().cloned().collect()
  }

  pub fn get_child_ref(&self, name: &str) -> Option<ActorRef<AnyMessage>> {
    let inner = self.inner.lock().unwrap();
    inner.children.get(name).and_then(|state| match state {
      ChildState::ChildRestartStats(stats) => Some(stats.child_ref().clone()),
      _ => None,
    })
  }

  pub fn get_child_state_by_ref(&self, actor_ref: ActorRef<AnyMessage>) -> Option<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner
      .children
      .values()
      .find(|state| match state {
        ChildState::ChildRestartStats(stats) => stats.child_ref() == &actor_ref,
        _ => false,
      })
      .cloned()
  }

  pub fn get_child_state_by_name(&self, name: &str) -> Option<ChildState> {
    let inner = self.inner.lock().unwrap();
    inner.children.get(name).cloned()
  }

  pub fn reserve_child(&mut self, name: &str) -> bool {
    let mut inner = self.inner.lock().unwrap();
    if inner.reserved_names.contains(name) {
      false
    } else {
      inner.reserved_names.insert(name.to_string());
      inner.children.insert(name.to_string(), ChildState::ChildNameReserved);
      true
    }
  }

  pub fn un_reserve_child(&mut self, name: &str) -> bool {
    let mut inner = self.inner.lock().unwrap();
    if inner.reserved_names.contains(name) {
      inner.reserved_names.remove(name);
      inner.children.remove(name);
      true
    } else {
      false
    }
  }

  pub fn init_child(&mut self, actor_ref: ActorRef<AnyMessage>, name: &str) -> Option<ChildState> {
    let result = {
      let inner = self.inner.lock().unwrap();
      inner.children.get(name).cloned()
    };
    let r = result.and_then(|state| match state {
      ChildState::ChildNameReserved => {
        let new_state = ChildState::ChildRestartStats(ChildRestartStats::new(actor_ref));
        let mut inner = self.inner.lock().unwrap();
        if let Some(one) = inner.children.get_mut(name) {
          *one = new_state.clone();
        } else {
          inner.children.insert(name.to_string(), new_state.clone());
        }
        Some(new_state)
      }
      ChildState::ChildRestartStats(stats) => {
        let new_state = ChildState::ChildRestartStats(stats.clone_with_new_child(actor_ref));
        let mut inner = self.inner.lock().unwrap();
        inner.children.insert(name.to_string(), new_state.clone());
        Some(new_state)
      }
    });
    r
  }

  fn make_child<U: Message>(
    &mut self,
    mut cell: ActorCell<AnyMessage>,
    self_ref: ActorRef<AnyMessage>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    // log::debug!("make_child: start: name = {}, children = {}", name, self_ref.path());
    self.reserve_child(name);
    let mut actor_ref = cell.new_child_actor(self_ref, props, name);
    self.init_child(actor_ref.clone().to_any(false), name).unwrap();
    // log::debug!("children: {:?}", self.children());
    actor_ref.start();
    // log::debug!(
    //   "make_child: finished: name = {}, children = {:?}",
    //   name,
    //   self.inner.lock().unwrap()
    // );
    actor_ref
  }

  pub fn actor_with_name_of<U: Message>(
    &mut self,
    cell: ActorCell<AnyMessage>,
    self_ref: ActorRef<AnyMessage>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    log::debug!("actor_with_name_of: {}", name);
    self.make_child(cell, self_ref, props, &Self::check_name(Some(name)))
  }

  pub fn actor_of<U: Message>(
    &mut self,
    cell: ActorCell<AnyMessage>,
    self_ref: ActorRef<AnyMessage>,
    props: Rc<dyn Props<U>>,
  ) -> ActorRef<U> {
    self.make_child(cell, self_ref, props, &Self::random_name())
  }

  fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    let factory = Base64StringFactory::new(true, false);
    let base64_string = factory.encode_from_bytes(&value.to_be_bytes());
    base64_string.to_string()
  }

  fn check_name(name: Option<&str>) -> String {
    match name {
      None => panic!("actor name must not be empty"),
      Some("") => panic!("actor name must not be empty"),
      Some(n) => {
        ActorPath::validate_path_element(n);
        n.to_string()
      }
    }
  }
}
