use std::sync::Arc;

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef, UntypedActorRef};
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct DefaultUntypedActorRef {
  actor_cell: ActorCell,
}

pub type Sender = Option<DefaultUntypedActorRef>;

impl DefaultUntypedActorRef {
  pub fn new(actor_cell: ActorCell) -> Self {
    Self { actor_cell }
  }
}

impl ActorRef for DefaultUntypedActorRef {
  fn name(&self) -> &str {
    self.path().name()
  }

  fn path(&self) -> &ActorPath {
    self.actor_cell.path()
  }
}

impl UntypedActorRef for DefaultUntypedActorRef {
  fn tell(&self, msg: AnyMessage, sender: Sender) {
    self
      .actor_cell
      .mailbox()
      .try_enqueue_any(msg, sender)
      .unwrap();
  }
}

impl InternalActorRef for DefaultUntypedActorRef {
  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    todo!()
  }

  fn start(&self) {
    todo!()
  }

  fn resume(&self) {
    todo!()
  }

  fn suspend(&self) {
    todo!()
  }

  fn stop(&self) {
    todo!()
  }

  fn tell_for_system(&self, message: SystemMessage) {
    todo!()
  }

  fn parent(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn get_child(&self, name: Vec<String>) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn is_local(&self) -> bool {
    true
  }

  fn is_terminated(&self) -> bool {
    todo!()
  }
}
