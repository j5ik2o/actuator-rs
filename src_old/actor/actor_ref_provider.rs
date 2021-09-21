use std::sync::Arc;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::InternalActorRef;
use crate::actor::address::Address;

pub type ActorRefProviderArc = Arc<dyn ActorRefProvider>;

pub trait ActorRefProvider {
  fn root_guardian(&self) -> Arc<dyn InternalActorRef>;
  fn root_path(&self) -> ActorPath;
}

pub struct LocalActorRefProvider {
  system_name: String,
}

impl LocalActorRefProvider {
  pub fn new(system_name: String) -> Self {
    Self { system_name }
  }
}

impl ActorRefProvider for LocalActorRefProvider {
  fn root_guardian(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn root_path(&self) -> ActorPath {
    let address = Address::from(("akka".to_string(), self.system_name.clone()));
    ActorPath::of_root(address)
  }
}
