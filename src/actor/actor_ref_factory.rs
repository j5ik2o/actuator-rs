use std::fmt::Debug;
use std::sync::Arc;

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor_system::{ActorSystemArc};

pub trait ActorRefFactory: Debug + Send + Sync {
  fn system(&self) -> ActorSystemArc;
  fn provider(&self) -> Arc<dyn ActorRefProvider>;
  // dispatcher()
  fn guardian(&self) -> Arc<dyn InternalActorRef>;
  fn lookup_root(&self) -> Arc<dyn InternalActorRef>;

  fn actor_of(&self) -> Arc<dyn ActorRef>;
  fn stop(&self, actor_ref: Arc<dyn ActorRef>);
}

pub trait TypedActorRefFactory: Debug + Send + Sync {}
