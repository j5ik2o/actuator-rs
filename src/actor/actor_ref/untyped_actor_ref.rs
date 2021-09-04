use std::sync::Arc;

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef, UntypedActorRef};
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor_system::ActorSystem;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct LocalActorRef {
  actor_cell: ActorCell,
}

pub type Sender = Option<LocalActorRef>;

impl LocalActorRef {
  // pub fn new(system: Arc<dyn ActorSystem>) -> Self {
  // let actor_cell =  ActorCell::new(system, )
  // Self { actor_cell }
  // }
}

impl ActorRef for LocalActorRef {
  fn name(&self) -> &str {
    self.path().name()
  }

  fn path(&self) -> &ActorPath {
    todo!()
//     self.actor_cell.path()
  }
}

impl UntypedActorRef for LocalActorRef {
  fn tell(self: Arc<Self>, msg: AnyMessage, sender: Sender) {
    todo!()
    // self
    //   .actor_cell
    //   .mailbox()
    //   .try_enqueue_any(msg, sender)
    //   .unwrap();
  }
}

impl InternalActorRef for LocalActorRef {
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
