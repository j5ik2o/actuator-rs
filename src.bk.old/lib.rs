use thiserror::Error;

use crate::actor::actor_ref::local_actor_ref_ref::LocalActorRefRef;
use crate::dispatch::any_message::AnyMessage;

pub mod actor;
pub mod dispatch;
pub mod infrastructure;

#[derive(Error, Debug, Clone)]
pub enum ActuatorError {
  #[error("Actor initialization error: {message}")]
  ActorInitializationError {
    actor: LocalActorRefRef<AnyMessage>,
    message: String,
    case: Option<Box<ActuatorError>>,
  },
}

impl PartialEq for ActuatorError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (
        ActuatorError::ActorInitializationError { actor, message, case },
        ActuatorError::ActorInitializationError {
          actor: other_actor,
          message: other_message,
          case: other_case,
        },
      ) => actor == other_actor && message == other_message && case == other_case,
    }
  }
}

impl ActuatorError {
  pub fn of_actor_initialization_error(
    actor: LocalActorRefRef<AnyMessage>,
    message: String,
    case: Option<Box<ActuatorError>>,
  ) -> Self {
    ActuatorError::ActorInitializationError { actor, message, case }
  }
}
