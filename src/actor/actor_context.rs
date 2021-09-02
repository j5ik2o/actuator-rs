use crate::actor::actor_ref::ActorRef;
use std::sync::Arc;
use crate::kernel::message::Message;

pub trait ActorContext<M: Message> {
  fn my_self(&self) -> Arc<dyn ActorRef>;
  fn parent(&self) -> Arc<dyn ActorRef>;
}
