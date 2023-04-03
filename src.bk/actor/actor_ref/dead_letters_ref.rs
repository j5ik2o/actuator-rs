use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::ActorRefBehavior;
use crate::dispatch::any_message::AnyMessage;

#[derive(Debug, Clone)]
pub struct DeadLettersRef {
  path: ActorPath,
}

impl DeadLettersRef {
  pub fn new(path: ActorPath) -> Self {
    Self { path }
  }
}

impl ActorRefBehavior for DeadLettersRef {
  type M = AnyMessage;

  fn path(&self) -> ActorPath {
    self.path.clone()
  }

  fn tell(&mut self, _msg: Self::M) {
    todo!()
  }
}
