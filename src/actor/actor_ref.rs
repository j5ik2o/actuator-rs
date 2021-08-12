use crate::kernel::Message;

#[derive(Clone)]
pub struct ActorRef<M: Message> {
  pub cell: ExtendedCell<M>,
}
