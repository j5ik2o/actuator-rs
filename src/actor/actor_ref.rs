use crate::actor::actor_path::ActorPath;

#[derive(PartialOrd)]
pub enum ActorRef<M> {
  Local(LocalActorRef<M>),
}

pub trait ActorRefBehavior: PartialOrd {
  type Message;
  fn path(&self) -> &ActorPath;
  fn tell(&self, msg: Self::Message);
}

pub trait InternalActorRefBehavior: ActorRefBehavior {
  type Message;
  fn start(&self);
  fn resume(&self);
  fn suspend(&self);
  fn restart<F>(&self, panic_f: F)
  where
    F: Fn();
  fn stop(&self);
  //   fn send_system_message
}

impl<M> ActorRefBehavior for ActorRef<M> {
  type Message = M;

  fn path(&self) -> &ActorPath {
    match self {
      ActorRef::Local(inner) => &inner.path,
    }
  }

  fn tell(&self, msg: Self::Message) {
    todo!()
  }
}

impl<M> ActorRef<M> {
  pub fn of_local(path: ActorPath) -> Self {
    ActorRef::Local(LocalActorRef::new(path))
  }
}

pub struct LocalActorRef<M> {
  path: ActorPath,
}

impl<M> LocalActorRef<M> {
  pub fn new(path: ActorPath) -> Self {
    Self { path }
  }
}
