use crate::actor_system::ActorSystem;
use crate::actor::actor_ref_provider::ActorRefProvider;
use std::sync::Arc;

pub trait ActorRefFactory {
  fn system(&self) -> Arc<dyn ActorSystem>;
  fn provider(&self) -> Arc<dyn ActorRefProvider>;
  // dispatcher()
}
