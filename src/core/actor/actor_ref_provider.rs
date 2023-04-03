use crate::core::actor::actor_path::ActorPath;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::address::Address;
use crate::core::dispatch::any_message::AnyMessage;
use std::fmt::Debug;

pub trait ActorRefProvider: Debug {
  fn root_guardian(&self) -> ActorRef<AnyMessage>;
  fn root_guardian_at(&self, address: Address) -> ActorRef<AnyMessage>;
  fn guardian(&self) -> ActorRef<AnyMessage>;
  fn system_guardian(&self) -> ActorRef<AnyMessage>;
  fn dead_letters(&self) -> ActorRef<AnyMessage>;
  fn root_path(&self) -> ActorPath;
}
