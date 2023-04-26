use crate::core::actor::actor_ref::local_actor_ref::LocalActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use thiserror::Error;

#[allow(dead_code)]
pub mod core;
pub mod infrastructure;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ActuatorError {
  #[error("Actor initialization error: {message}")]
  ActorInitializationError {
    actor: LocalActorRef<AnyMessage>,
    message: String,
    case: Option<Box<ActuatorError>>,
  },
}
