use std::sync::Arc;

use crate::actor::actor_context::ActorContext;
use crate::actor::actor_ref::InternalActorRef;
use crate::actor_system::ActorSystem;

pub trait Cell {
  fn system(&self) -> Arc<dyn ActorSystem>;
  fn start(&self) -> Arc<dyn ActorContext>;
  fn suspend(&self);
  fn resume(panic_by_failure: &str);
  fn restart(panic_message: &str);
  fn stop(&self);

  fn parent(&self) -> Arc<dyn InternalActorRef>;
}
