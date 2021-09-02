use crate::actor::actor_ref_provider::ActorRefProvider;
use std::sync::Arc;
use std::fmt::Debug;

pub trait ActorSystem: Debug + Send + Sync {
  fn provider(&self) -> Arc<dyn ActorRefProvider>;
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DefaultActorSystem {
  debug: bool,
}

impl ActorSystem for DefaultActorSystem {
  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    todo!()
  }
}
