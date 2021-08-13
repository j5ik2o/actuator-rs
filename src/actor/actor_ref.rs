use crate::actor::extended_cell::ExtendedCell;
use crate::kernel::{Envelope, Message};

#[derive(Clone)]
pub struct ActorRef<M: Message> {
  pub cell: ExtendedCell<M>,
}

impl<M: Message> ActorRef<M> {
  pub fn new(cell: ExtendedCell<M>) -> ActorRef<M> {
    Self { cell }
  }

  pub fn send_message(&self, message: M) {
    let _envelope = Envelope::new(message);
    // let _ = self.cell.send_message(envelope);
  }
}
