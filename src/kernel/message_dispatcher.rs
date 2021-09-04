use std::marker::PhantomData;
use std::time::Duration;

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;
use crate::actor::actor_ref::ToActorRef;

pub trait MessageDispatcherConfigurator {}

pub trait MessageDispatcher {
  fn id(&self) -> &str;
  fn throughput(&self) -> u32;
  fn throughput_deadline_time(&self) -> Duration;
  fn shutdown_timeout(&self) -> Duration;
  fn dispatch(&self, receiver: &ActorCell, invocation: AnyMessage, sender: Sender);
}

pub struct Dispatcher {
  id: String,
  throughput: u32,
  throughput_deadline_time: Duration,
  shutdown_timeout: Duration,
}

impl Dispatcher {
  // fn dispatch(&self, receiver: &ActorCell, invocation: Envelope<M>) {
  //     let mbox = receiver.mailbox();
  //     mbox.try_enqueue(receiver.my_self, invocation)
  // }

  fn suspend(&self, actor: &ActorCell) {
    let mbox = actor.mailbox();
    //    mbox.actor() == actor && mbox.dispatcher() == self;
    //.suspend();
  }
}

impl MessageDispatcher for Dispatcher {
  fn id(&self) -> &str {
    &self.id
  }

  fn throughput(&self) -> u32 {
    self.throughput
  }

  fn throughput_deadline_time(&self) -> Duration {
    self.throughput_deadline_time
  }

  fn shutdown_timeout(&self) -> Duration {
    self.shutdown_timeout
  }

  fn dispatch(&self, receiver: &ActorCell, invocation: AnyMessage, sender: Sender) {
    let mbox = receiver.mailbox();
    let receiver = mbox.actor();
    let ref_ = receiver.my_self();
    mbox
      .try_enqueue_any(ref_.unwrap().to_actor_ref(), invocation, sender)
      .unwrap();
  }
}
