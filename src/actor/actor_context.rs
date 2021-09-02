use std::sync::Arc;

use crate::actor::actor_ref::ActorRef;
use crate::actor::actor_ref_factory::ActorRefFactory;
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::kernel::message::Message;

pub trait ActorContext: ActorRefFactory {
  fn self_ref(&self) -> Arc<dyn ActorRef>;
  fn parent_ref(&self) -> Arc<dyn ActorRef>;
}
