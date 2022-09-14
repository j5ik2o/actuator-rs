use std::cell::RefCell;
use std::rc::Rc;

use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::props::Props;
use crate::dispatch::message::Message;

pub trait ActorContextBehavior<Msg: Message> {
  fn my_self(&self) -> ActorRefRef<Msg>;
  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRefRef<U>;
  fn stop<U: Message>(child: ActorRefRef<U>);
}

#[derive(Debug, Clone)]
pub struct ActorContext<Msg: Message> {
  actor_cell: ActorCellRef<Msg>,
}

#[derive(Debug, Clone)]
pub struct ActorContextRef<Msg: Message> {
  inner: Rc<RefCell<ActorContext<Msg>>>,
}

impl<Msg: Message> ActorContextRef<Msg> {
  pub fn new(actor_cell: ActorCellRef<Msg>) -> Self {
    Self {
      inner: Rc::new(RefCell::new(ActorContext { actor_cell })),
    }
  }
}

impl<Msg: Message> ActorContextBehavior<Msg> for ActorContextRef<Msg> {
  fn my_self(&self) -> ActorRefRef<Msg> {
    let inner = self.inner.borrow();
    inner.actor_cell.actor_ref_ref()
  }

  fn spawn<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRefRef<U> {
    let mut actor_cell = {
      let inner = self.inner.borrow();
      inner.actor_cell.clone()
    };

    let actor_ref = actor_cell.new_actor_ref(props, name);
    actor_cell.init_child(actor_ref.clone());
    actor_ref
  }

  fn stop<U: Message>(_child: ActorRefRef<U>) {
    todo!()
  }
}
