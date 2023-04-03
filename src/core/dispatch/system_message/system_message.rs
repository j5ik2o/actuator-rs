use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::ActorError;
use crate::core::dispatch::any_message::AnyMessage;
use crate::ActuatorError;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemMessage {
  Create {
    failure: Option<ActuatorError>,
  },
  Recreate {
    cause: ActorError,
  },
  Suspend,
  Resume {
    caused_by_failure: Option<ActorError>,
  },
  Terminate,
  Supervise {
    child: ActorRef<AnyMessage>,
    a_sync: bool,
  },
  Watch,
  NoMessage,
  Failed {
    child: ActorRef<AnyMessage>,
    error: ActorError,
    uid: u32,
  },
  DeathWatchNotification {
    actor: ActorRef<AnyMessage>,
    existence_confirmed: bool,
    address_terminated: bool,
  },
}

impl SystemMessage {
  pub fn of_create() -> Self {
    SystemMessage::Create { failure: None }
  }

  pub fn of_create_with_failure(failure: Option<ActuatorError>) -> Self {
    SystemMessage::Create { failure }
  }

  pub fn of_recreate(cause: ActorError) -> Self {
    SystemMessage::Recreate { cause }
  }

  pub fn of_suspend() -> Self {
    SystemMessage::Suspend
  }

  pub fn of_resume() -> Self {
    SystemMessage::Resume {
      caused_by_failure: None,
    }
  }

  pub fn of_resume_with_failure(caused_by_failure: Option<ActorError>) -> Self {
    SystemMessage::Resume { caused_by_failure }
  }

  pub fn of_terminate() -> Self {
    SystemMessage::Terminate
  }

  pub fn of_supervise(child: ActorRef<AnyMessage>, a_sync: bool) -> Self {
    SystemMessage::Supervise { child, a_sync }
  }

  pub fn of_watch() -> Self {
    SystemMessage::Watch
  }

  pub fn of_no_message() -> Self {
    SystemMessage::NoMessage
  }

  pub fn of_failed(child: ActorRef<AnyMessage>, error: ActorError, uid: u32) -> Self {
    SystemMessage::Failed { child, error, uid }
  }

  pub fn of_death_watch_notification(
    actor: ActorRef<AnyMessage>,
    existence_confirmed: bool,
    address_terminated: bool,
  ) -> Self {
    SystemMessage::DeathWatchNotification {
      actor,
      existence_confirmed,
      address_terminated,
    }
  }

  pub fn is_no_message(&self) -> bool {
    match self {
      SystemMessage::NoMessage { .. } => true,
      _ => false,
    }
  }
}
