use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_ref::{InternalActorRef, ActorRef};
use crate::actor::actor_path::ActorPath;
use crate::kernel::system_message::SystemMessage;
use std::sync::Arc;
use crate::actor::actor_ref_provider::ActorRefProvider;

#[derive(Debug, Clone)]
pub struct UntypedActorRef {
  cell: ActorCell,
}

pub type Sender = Option<UntypedActorRef>;

impl UntypedActorRef {
  pub fn new(cell: ActorCell) -> Self {
    Self { cell }
  }
}

impl ActorRef for UntypedActorRef {
  fn path(&self) -> &ActorPath {
    self.cell.path()
  }
}

impl InternalActorRef for UntypedActorRef {
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

  fn send_message_for_system(&self, message: SystemMessage) {
    todo!()
  }

  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    todo!()
  }

  fn parent(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn is_local(&self) -> bool {
    true
  }

  fn is_terminated(&self) -> bool {
    todo!()
  }
}