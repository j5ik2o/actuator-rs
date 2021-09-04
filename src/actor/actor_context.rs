use std::sync::Arc;

use crate::actor::actor_ref::UntypedActorRef;
use crate::actor::actor_ref_factory::ActorRefFactory;

use crate::actor_system::ActorSystem;

use crate::actor::children::Children;

pub trait ActorContext: ActorRefFactory {
  fn self_ref(&self) -> Arc<dyn UntypedActorRef>;
  fn parent_ref(&self) -> Arc<dyn UntypedActorRef>;
  fn children(&self) -> Children;
  fn child(&self, name: &str) -> Option<Box<dyn UntypedActorRef>>;
  fn system(&self) -> Arc<dyn ActorSystem>;
}

pub trait TypedActorContext: ActorRefFactory {}
