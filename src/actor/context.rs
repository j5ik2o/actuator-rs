use crate::kernel::Message;
use crate::actor::actor_ref::ActorRef;

pub struct Context<M: Message> {
  pub my_self: ActorRef<M>,
  //    pub system: ActorSystem,
  //    pub(crate) kernel: KernelRef,
}
