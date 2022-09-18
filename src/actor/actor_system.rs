use crate::actor::actor_ref_factory::ActorRefFactory;

pub enum ActorSystem {
  ActorSystemImpl,
  ExtendedActorSystem,
}

impl ActorRefFactory for ActorSystem {}
