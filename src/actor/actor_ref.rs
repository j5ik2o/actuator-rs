use std::cmp::Ordering;
use std::sync::Arc;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor::extended_cell::ExtendedCell;
use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;
use crate::kernel::system_message::SystemMessage;
use std::fmt::Debug;

pub mod internal_actor_ref;
pub mod local_actor_ref;

pub trait ToActorRef {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a>
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

pub trait ActorRef: ToActorRef + Debug + Send + Sync {
  fn path(&self) -> &ActorPath;
}

pub trait InternalActorRef: ActorRef {
  fn start(&self);
  fn resume(&self);
  fn suspend(&self);
  fn stop(&self);

  fn send_message_for_system(&self, message: SystemMessage);

  fn provider(&self) -> Arc<dyn ActorRefProvider>;
  fn parent(&self) -> Arc<dyn InternalActorRef>;

  // fn get_child<I: Iterator>(&self, name: I) -> Arc<dyn InternalActorRef>;

  fn is_local(&self) -> bool;

  fn is_terminated(&self) -> bool;
}
