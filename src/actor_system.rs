use std::fmt::Debug;
use std::sync::Arc;

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::actor_ref_factory::ActorRefFactory;
use crate::actor::actor_ref_provider::{ActorRefProvider, LocalActorRefProvider};

pub type ActorSystemArc = Arc<dyn ActorSystem>;

pub trait ActorSystem: ActorRefFactory + Debug + Send + Sync {
  fn name(&self) -> &str;
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LocalActorSystem {
  debug: bool,
  name: String,
}

impl Default for LocalActorSystem {
  fn default() -> Self {
    Self::new(&String::default(), bool::default())
  }
}

impl LocalActorSystem {
  pub fn new(name: &str, debug: bool) -> Self {
    Self {
      debug,
      name: name.to_string(),
    }
  }
}

impl ActorRefFactory for LocalActorSystem {
  fn system(&self) -> Arc<dyn ActorSystem> {
    Arc::new(self.clone())
  }

  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    Arc::new(LocalActorRefProvider::new(self.name().to_string()))
  }

  fn guardian(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn lookup_root(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn actor_of(&self) -> Arc<dyn ActorRef> {
    todo!()
  }

  fn stop(&self, _actor_ref: Arc<dyn ActorRef>) {
    todo!()
  }
}

impl ActorSystem for LocalActorSystem {
  fn name(&self) -> &str {
    &self.name
  }
}
