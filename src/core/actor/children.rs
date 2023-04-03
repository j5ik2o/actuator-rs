use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use base64_string_rs::Base64StringFactory;
use rand::RngCore;

use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::actor::actor_ref_provider::ActorRefProvider;
use crate::core::actor::children::child_state::{ChildRestartStats, ChildState};
use crate::core::actor::children::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::core::actor::props::Props;
use crate::core::actor::{actor_cell, ActorError};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;

mod child_state;
mod children_container;

pub struct FunctionRef {
  actor_path: ActorPath,
  provider: Rc<dyn ActorRefProvider>,
  f: Rc<dyn Fn((ActorRef<AnyMessage>, AnyMessage))>,
}

impl Debug for FunctionRef {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FunctionRef")
      .field("actor_path", &self.actor_path)
      .finish()
  }
}

impl Clone for FunctionRef {
  fn clone(&self) -> Self {
    Self {
      actor_path: self.actor_path.clone(),
      provider: self.provider.clone(),
      f: self.f.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Children {
  container: Arc<RwLock<ChildrenContainer>>,
  terminated: Arc<Mutex<AtomicBool>>,
  // function_ref: Arc<Mutex<HashMap<String, FunctionRef>>>,
}

impl Children {
  fn swap_children(
    &mut self,
    old_container: Arc<RwLock<ChildrenContainer>>,
    new_container: Arc<RwLock<ChildrenContainer>>,
  ) -> bool {
    if Arc::ptr_eq(&self.container, &old_container) {
      self.container = new_container;
      true
    } else {
      let mut swap = false;
      {
        let ll = self.container.read().unwrap();
        let rr = old_container.read().unwrap();
        if *ll == *rr {
          swap = true;
        }
      }
      if swap {
        self.container = new_container;
      }
      swap
    }
  }

  pub fn new() -> Self {
    Self {
      container: Arc::new(RwLock::new(ChildrenContainer::of_empty())),
      terminated: Arc::new(Mutex::new(AtomicBool::new(false))),
    }
  }

  pub fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    self.container.read().unwrap().children()
  }

  pub fn get_child_ref(&self, name: &str) -> Option<ActorRef<AnyMessage>> {
    match self.container.read().unwrap().get_by_name(name) {
      Some(child) => child.as_child_restart_stats().map(|e| e.child_ref().clone()),
      None => None,
    }
  }

  pub fn get_all_child_stats(&self) -> Vec<ChildState> {
    self.container.read().unwrap().get_all_child_stats()
  }

  pub fn get_single_child_ref(&self, name: &str) -> Option<ActorRef<AnyMessage>> {
    if !name.contains('#') {
      match self.get_child_state_by_name(name) {
        Some(child) => child.as_child_restart_stats().map(|e| e.child_ref().clone()),
        None => None,
      }
    } else {
      let (child_name, uid) = actor_cell::split_name_and_uid(name);
      match self.get_child_state_by_name(child_name) {
        Some(child) if uid == actor_cell::UNDEFINED_UID || uid == child.as_child_restart_stats().unwrap().uid() => {
          Some(child.as_child_restart_stats().unwrap().child_ref().clone())
        }
        _ => None,
      }
    }
  }

  pub fn get_child_state_by_ref(&self, actor_ref: ActorRef<AnyMessage>) -> Option<ChildState> {
    self.container.read().unwrap().get_by_ref(actor_ref)
  }

  pub fn get_child_state_by_name(&self, name: &str) -> Option<ChildState> {
    self.container.read().unwrap().get_by_name(name)
  }

  pub fn reserve_child(&mut self, name: &str) -> bool {
    let mut cc = self.container.read().unwrap().deep_copy();
    self.swap_children(self.container.clone(), Arc::new(RwLock::new(cc.reserve(name)))) || self.reserve_child(name)
  }

  pub fn un_reserve_child(&mut self, name: &str) -> bool {
    let mut cc = self.container.read().unwrap().deep_copy();
    self.swap_children(self.container.clone(), Arc::new(RwLock::new(cc.un_reserve(name))))
      || self.un_reserve_child(name)
  }

  pub fn stop(&mut self, mut actor_ref: ActorRef<AnyMessage>) {
    fn shall_die(myself: &mut Children, mut cc: ChildrenContainer, actor_ref: ActorRef<AnyMessage>) -> bool {
      myself.swap_children(
        myself.container.clone(),
        Arc::new(RwLock::new(cc.shall_die(actor_ref.clone()))),
      ) || shall_die(myself, cc, actor_ref)
    }
    let result = {
      let r = self.container.read().unwrap();
      r.get_by_ref(actor_ref.clone())
    };
    if result.is_some() {
      let r = self.container.read().unwrap().deep_copy();
      shall_die(self, r, actor_ref.clone());
    }
    actor_ref.stop();
  }

  pub fn init_child(&mut self, actor_ref: ActorRef<AnyMessage>) -> Option<ChildState> {
    let mut cc = self.container.read().unwrap().deep_copy();
    let path = actor_ref.path();
    match cc.get_by_name(path.name()) {
      old @ Some(ChildState::ChildRestartStats(..)) => old,
      Some(ChildState::ChildNameReserved) => {
        let crs = ChildState::ChildRestartStats(ChildRestartStats::new(actor_ref.clone()));
        if self.swap_children(
          self.container.clone(),
          Arc::new(RwLock::new(cc.add(path.name(), crs.clone()))),
        ) {
          Some(crs)
        } else {
          self.init_child(actor_ref)
        }
      }
      None => None,
    }
  }

  fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    let factory = Base64StringFactory::new(true, false);
    let base64_string = factory.encode_from_bytes(&value.to_be_bytes());
    base64_string.to_string()
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

  fn make_child<U: Message>(
    &mut self,
    mut cell: ActorCell<AnyMessage>,
    self_ref: ActorRef<AnyMessage>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    // log::debug!("make_child: {}: start", name);
    self.reserve_child(name);
    let mut actor_ref = cell.new_child_actor(self_ref, props, name);
    self.init_child(actor_ref.clone().to_any()).unwrap();
    actor_ref.start();
    // log::debug!("make_child: {}: finished", name);
    actor_ref
  }

  pub fn suspend_children(&mut self, except_for: Vec<ActorRef<AnyMessage>>) {
    let mut cc = self.container.write().unwrap();
    cc.stats_mut(|cs| match cs {
      ChildState::ChildRestartStats(crs) if !except_for.contains(crs.child_ref()) => {
        log::debug!("crs = {:?}", crs);
        crs.child_ref_mut().suspend()
      }
      _ => {}
    });
  }

  pub fn resume_children(&mut self, caused_by_failure: ActorError, perp: ActorRef<AnyMessage>) {
    let mut cc = self.container.write().unwrap();
    cc.stats_mut(|cs| match cs {
      ChildState::ChildRestartStats(crs) => {
        let child = crs.child_ref().clone();
        crs.child_ref_mut().as_local_mut().into_iter().for_each(|e| {
          e.resume(
            child.clone(),
            if perp == child {
              Some(caused_by_failure.clone())
            } else {
              None
            },
          )
        })
      }
      _ => {}
    });
  }

  pub fn set_children_termination_reason(&mut self, reason: SuspendReason) -> bool {
    let cc = self.container.read().unwrap().deep_copy();
    match cc.clone() {
      ChildrenContainer::Terminating(c) => {
        let mut ncc = c.deep_copy();
        ncc.set_reason(reason.clone());
        self.swap_children(
          Arc::new(RwLock::new(cc)),
          Arc::new(RwLock::new(ChildrenContainer::Terminating(ncc))),
        ) || self.set_children_termination_reason(reason)
      }
      _ => false,
    }
  }

  pub fn set_terminated(&mut self) {
    self
      .terminated
      .lock()
      .unwrap()
      .store(true, std::sync::atomic::Ordering::SeqCst);
  }

  pub fn is_normal(&self) -> bool {
    self.container.read().unwrap().is_normal()
  }

  pub fn is_terminating(&self) -> bool {
    self.container.read().unwrap().is_terminating()
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

#[cfg(test)]
mod tests {
  use crate::core::actor::actor_path::ActorPath;

  use super::*;

  #[test]
  fn test_reserve_child() {
    let mut children = Children::new();
    let actor_path = ActorPath::from_string("test://test/test");
    let result = children.reserve_child(actor_path.name());
    assert!(result);
  }

  #[test]
  fn test_init_child_is_node() {
    let mut children = Children::new();
    let actor_path = ActorPath::from_string("test://test/test");
    let actor_ref = ActorRef::Mock(actor_path.clone());
    let result = children.init_child(actor_ref);
    assert!(result.is_none());
  }

  #[test]
  fn test_init_child_is_some() {
    let mut children = Children::new();
    let actor_path = ActorPath::from_string("test://test/test");
    let actor_ref = ActorRef::Mock(actor_path.clone());
    children.reserve_child(actor_path.name());
    let result = children.init_child(actor_ref);
    assert!(result.is_some());
  }
}
