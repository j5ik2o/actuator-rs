use std::sync::Arc;

use dashmap::DashMap;

use crate::actor::actor_ref::{ActorRef, InternalActorRef, UntypedActorRef};
use crate::actor::actor_ref::untyped_actor_ref::LocalActorRef;

#[derive(Debug, Clone)]
pub struct Children {
  actors: Arc<DashMap<String, LocalActorRef>>,
}

impl Children {
  pub fn new() -> Self {
    Self {
      actors: Arc::new(DashMap::new()),
    }
  }

  pub fn add(&self, actor_ref: LocalActorRef) -> &Self {
    self.actors.insert(actor_ref.name().to_string(), actor_ref);
    self
  }

  pub fn remove(&self, actor_ref: &LocalActorRef) -> &Self {
    self.actors.remove(actor_ref.name());
    self
  }

  pub fn len(&self) -> usize {
    self.actors.len()
  }

  pub fn iter(&self) -> impl Iterator<Item = LocalActorRef> + '_ {
    self.actors.iter().map(|e| e.value().clone())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::actor::actor_cell::ActorCell;
  use crate::actor::actor_path::ActorPath;
  use crate::actor::actor_ref::untyped_actor_ref::LocalActorRef;
  use crate::actor::children::Children;
  use crate::actor_system::LocalActorSystem;
  use crate::kernel::{MailboxType, new_mailbox};

  #[test]
  fn test_add() {
    // let actor_system = Arc::new(LocalActorSystem::default());
    // let actor_path = ActorPath::from_string("tcp::/test@test/test");
    // let mailbox = new_mailbox(MailboxType::MPSC, 1);
    // let any_sender = Arc::new(mailbox.new_sender());
    // let system_sender = mailbox.new_system_sender();
    //
    // let actor_cell = ActorCell::new(actor_system, actor_path, any_sender, system_sender);
    // let children = Children::new().add(LocalActorRef::new(actor_cell));
  }
}
