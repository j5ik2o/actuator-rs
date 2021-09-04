use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::message::Message;
use crate::kernel::system_message::SystemMessage;

pub mod typed_actor_ref;
pub mod untyped_actor_ref;

pub trait ToActorRef {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a>
  where
    Self: 'a;
}

pub trait ToUntypedActorRef {
  fn to_untyped_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn UntypedActorRef + 'a>
  where
    Self: 'a;
}

pub trait ToTypedActorRef<M: Message> {
  fn to_typed_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn TypedActorRef<M> + 'a>
  where
    Self: 'a;
}

impl<T: InternalActorRef> ToActorRef for T {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a>
  where
    Self: 'a,
  {
    self
  }
}

impl<T: InternalActorRef> ToUntypedActorRef for T {
  fn to_untyped_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn UntypedActorRef + 'a>
  where
    Self: 'a,
  {
    self
  }
}

pub trait ActorRef: ToActorRef + Debug + Send + Sync {
  fn name(&self) -> &str;
  fn path(&self) -> &ActorPath;
}

pub trait UntypedActorRef: ToUntypedActorRef + ActorRef {
  fn tell(self: Arc<Self>, msg: AnyMessage, sender: Sender);
}

pub trait InternalActorRef: UntypedActorRef {
  fn provider(&self) -> Arc<dyn ActorRefProvider>;

  fn start(&self);
  fn resume(&self);
  fn suspend(&self);
  fn stop(&self);

  fn tell_for_system(&self, message: SystemMessage);

  fn parent(&self) -> Arc<dyn InternalActorRef>;
  fn get_child(&self, name: Vec<String>) -> Arc<dyn InternalActorRef>;
  fn is_local(&self) -> bool;
  fn is_terminated(&self) -> bool;
}

pub trait TypedActorRef<M: Message>: ActorRef {
  fn tell(self: Arc<Self>, message: M);
}
