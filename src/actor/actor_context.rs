use std::sync::Arc;

use crate::actor::actor_ref::UntypedActorRef;
use crate::actor::actor_ref_factory::ActorRefFactory;
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor_system::ActorSystem;
use crate::kernel::message::Message;

pub trait ActorContext: ActorRefFactory {
  fn self_ref(&self) -> Arc<dyn UntypedActorRef>;
  fn parent_ref(&self) -> Arc<dyn UntypedActorRef>;
  fn children(&self) -> Vec<Arc<dyn UntypedActorRef>>;
  fn child(&self, name: &str) -> Option<Arc<dyn UntypedActorRef>>;
  fn system(&self) -> Arc<dyn ActorSystem>;
}
