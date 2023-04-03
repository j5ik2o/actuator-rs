use crate::actor::actor::{Actor, ActorMutableBehavior, ActorResult};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use std::time::Duration;
use ulid_generator_rs::ULIDGenerator;

use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_cell::{ActorCellBehavior, ActorCellInnerBehavior};
use crate::actor::actor_path::ActorPathBehavior;

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior};
use crate::actor::props::{Props, SingletonProps};
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::message::Message;

pub trait ActorContextBehavior<Msg: Message> {
  fn self_ref(&self) -> ActorRefRef<Msg>;
  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRefRef<U>;
  fn stop<U: Message>(&mut self, child: ActorRefRef<U>);
  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg);
  fn cancel_receive_timeout(&mut self);
  fn get_receive_timeout(&self) -> Option<Duration>;
  fn message_adaptor<U: Message>(&self, f: impl Fn(U) -> Msg + 'static) -> ActorRefRef<U>;
}

#[derive(Debug, Clone)]
pub struct ActorContext<Msg: Message> {
  actor_cell: ActorCellRef<Msg>,
  actor_ref: ActorRefRef<Msg>,
}

#[derive(Debug, Clone)]
pub struct ActorContextRef<Msg: Message> {
  inner: Rc<RefCell<ActorContext<Msg>>>,
}

impl<Msg: Message> ActorContextRef<Msg> {
  pub fn new(actor_cell: ActorCellRef<Msg>, actor_ref: ActorRefRef<Msg>) -> Self {
    Self {
      inner: Rc::new(RefCell::new(ActorContext { actor_cell, actor_ref })),
    }
  }
}

impl<Msg: Message> ActorContextBehavior<Msg> for ActorContextRef<Msg> {
  fn self_ref(&self) -> ActorRefRef<Msg> {
    let inner = self.inner.borrow();
    inner.actor_ref.clone()
  }

  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRefRef<U> {
    let mut actor_cell = {
      let inner = self.inner.borrow();
      inner.actor_cell.clone()
    };
    let actor_ref = actor_cell.new_child_actor_ref(self.self_ref(), props, name);
    actor_cell.init_child(actor_ref.clone());
    actor_ref
  }

  fn stop<U: Message>(&mut self, child: ActorRefRef<U>) {
    child.as_local().unwrap().stop();
  }

  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg) {
    let mut inner = self.inner.borrow_mut();
    inner.actor_cell.set_receive_timeout(timeout, msg);
  }

  fn cancel_receive_timeout(&mut self) {
    let mut inner = self.inner.borrow_mut();
    inner.actor_cell.cancel_receive_timeout();
  }

  fn get_receive_timeout(&self) -> Option<Duration> {
    let inner = self.inner.borrow_mut();
    inner.actor_cell.get_receive_timeout()
  }

  fn message_adaptor<U: Message>(&self, f: impl Fn(U) -> Msg + 'static) -> ActorRefRef<U> {
    let inner = self.inner.borrow();
    let binding = inner.actor_cell.parent_ref();
    let parent_ref = binding.as_ref().unwrap();
    adaptor(f, self.self_ref(), parent_ref.clone())
  }
}

fn adaptor<In: Message, Out: Message>(
  f: impl Fn(In) -> Out + 'static,
  underlying: ActorRefRef<Out>,
  parent_ref: ActorRefRef<AnyMessage>,
) -> ActorRefRef<In> {
  let ulid = ULIDGenerator::new().generate().unwrap().to_string();
  let local_actor_ref_ref = underlying.as_local().unwrap();
  let actor_system_context = local_actor_ref_ref.actor_system_context();
  let mailbox_type = local_actor_ref_ref.mailbox_type();
  let child_name = format!("wrapper-{}", ulid);
  let actor_path = parent_ref.path().with_child(&child_name);

  let wrapper = Actor::of_mutable(Rc::new(RefCell::new(MessageAdaptor::new(f, underlying))));

  let mut actor_ref_ref = ActorRefRef::of_local(actor_path);
  let props = Rc::new(SingletonProps::new(wrapper));
  actor_ref_ref.initialize(actor_system_context, mailbox_type, props, Some(parent_ref));
  actor_ref_ref
}

struct MessageAdaptor<In: Message, Out: Message> {
  f: Rc<dyn Fn(In) -> Out>,
  underlying: ActorRefRef<Out>,
}

impl<In: Message, Out: Message> MessageAdaptor<In, Out> {
  fn new(f: impl Fn(In) -> Out + 'static, underlying: ActorRefRef<Out>) -> Self {
    Self {
      f: Rc::new(f),
      underlying,
    }
  }
}

impl<In: Message, Out: Message> Debug for MessageAdaptor<In, Out> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MessageAdaptor")
      .field("f", &"f<*>")
      .field("underlying", &self.underlying)
      .finish()
  }
}

impl<In: Message, Out: Message> Clone for MessageAdaptor<In, Out> {
  fn clone(&self) -> Self {
    Self {
      f: self.f.clone(),
      underlying: self.underlying.clone(),
    }
  }
}

impl<In: Message, Out: Message> ActorMutableBehavior for MessageAdaptor<In, Out> {
  type Msg = In;

  fn receive(&mut self, mut ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()> {
    self.underlying.tell((self.f)(msg));
    ctx.stop(ctx.self_ref());
    Ok(())
  }
}
