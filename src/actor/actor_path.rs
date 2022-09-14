use crate::actor::address::Address;

pub trait ActorPath: PartialOrd {
  fn address() -> Address;
}
pub struct ActorPathImpl {}
