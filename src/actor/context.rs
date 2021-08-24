use crate::actor::actor_ref::ActorRef;
use crate::kernel::message::Message;

pub struct Context<M: Message> {
  pub my_self: ActorRef<M>,
  //    pub system: ActorSystem,
  //    pub(crate) kernel: KernelRef,
}
