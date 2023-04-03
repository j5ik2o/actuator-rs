use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use std::rc::Rc;
use std::time::Duration;

pub struct ActorContext<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  actor_cell: ActorCellWithRef<Msg>,
}

impl<Msg: Message> ActorContext<Msg> {
  pub(crate) fn new(actor_cell: ActorCell<Msg>, self_ref: ActorRef<Msg>) -> Self {
    Self {
      _phantom: std::marker::PhantomData,
      actor_cell: ActorCellWithRef::new(actor_cell, self_ref),
    }
  }
}

impl ActorContext<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> ActorContext<Msg> {
    ActorContext {
      _phantom: std::marker::PhantomData,
      actor_cell: ActorCellWithRef::new(
        self.actor_cell.actor_cell.to_typed(),
        self.actor_cell.actor_ref.to_typed(),
      ),
    }
  }
}

pub trait ActorContextBehavior<Msg: Message> {
  fn self_ref(&self) -> ActorRef<Msg>;
  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRef<U>;
  fn stop<U: Message>(&mut self, child: ActorRef<U>);
  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg);
  fn cancel_receive_timeout(&mut self);
  fn get_receive_timeout(&self) -> Option<Duration>;
  fn message_adaptor<U: Message>(&self, f: impl Fn(U) -> Msg + 'static) -> ActorRef<U>;
}

impl<Msg: Message> ActorContextBehavior<Msg> for ActorContext<Msg> {
  fn self_ref(&self) -> ActorRef<Msg> {
    self.actor_cell.actor_ref.clone()
  }

  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRef<U> {
    log::debug!("spawn: {}", name);
    self.actor_cell.actor_with_name_of(props, name)
  }

  fn stop<U: Message>(&mut self, mut child: ActorRef<U>) {
    child.stop();
  }

  fn set_receive_timeout(&mut self, _timeout: Duration, _msg: Msg) {
    todo!()
  }

  fn cancel_receive_timeout(&mut self) {
    todo!()
  }

  fn get_receive_timeout(&self) -> Option<Duration> {
    todo!()
  }

  fn message_adaptor<U: Message>(&self, _f: impl Fn(U) -> Msg + 'static) -> ActorRef<U> {
    todo!()
  }
}
