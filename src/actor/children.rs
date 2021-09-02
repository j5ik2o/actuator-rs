use std::sync::Arc;
use dashmap::DashMap;
use crate::actor::actor_ref::{UntypedActorRef, InternalActorRef, ActorRef};
use crate::actor::actor_ref::untyped_actor_ref::DefaultUntypedActorRef;
#[derive(Debug, Clone)]
pub struct Children {
  actors: Arc<DashMap<String, DefaultUntypedActorRef>>,
}

impl Children {
  pub fn new() -> Self {
    Self {
      actors: Arc::new(DashMap::new()),
    }
  }

  pub fn add(&self, actor_ref: DefaultUntypedActorRef) {
    self.actors.insert(actor_ref.name().to_string(), actor_ref);
  }

  pub fn remove(&self, actor_ref: &DefaultUntypedActorRef) {
    self.actors.remove(actor_ref.name());
  }

  pub fn len(&self) -> usize {
    self.actors.len()
  }

  pub fn iter(&self) -> impl Iterator<Item = DefaultUntypedActorRef> + '_ {
    self.actors.iter().map(|e| e.value().clone())
  }
}
