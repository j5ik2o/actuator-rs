use std::marker::PhantomData;

use crate::actor::actor_cell::ActorCell;
use crate::kernel::message::Message;

pub struct MessageDispatcher<M: Message> {
  _phantom_data: PhantomData<M>,
}

impl<M: Message> MessageDispatcher<M> {
  // fn dispatch(&self, receiver: &ActorCell, invocation: Envelope<M>) {
  //     let mbox = receiver.mailbox();
  //     mbox.try_enqueue(receiver.my_self, invocation)
  // }

  fn suspend(&self, actor: &ActorCell) {
    actor.mailbox().suspend();
  }
}
