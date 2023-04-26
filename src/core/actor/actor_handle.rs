use crate::core::actor::{ActorBehavior, AnyMessageActorWrapper};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct ActorHandleInner<Msg: Message> {
  actor: Option<Rc<RefCell<dyn ActorBehavior<Msg>>>>,
}

#[derive(Debug, Clone)]
pub struct ActorHandle<Msg: Message> {
  inner: Rc<RefCell<ActorHandleInner<Msg>>>,
}

impl ActorHandle<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> ActorHandle<Msg> {
    let inner_actor = {
      let inner = self.inner.borrow();
      if let Some(actor) = &inner.actor {
        let result = {
          let ptr = Rc::into_raw(actor.clone());
          let raw_ptr_cast = ptr as *const RefCell<AnyMessageActorWrapper<Msg>>;
          let rc = unsafe { Rc::from_raw(raw_ptr_cast) };
          let b = rc.borrow();
          b.inner_actor.clone()
        };
        Some(result)
      } else {
        None
      }
    };
    ActorHandle {
      inner: Rc::new(RefCell::new(ActorHandleInner { actor: inner_actor })),
    }
  }
}

impl<Msg: Message> ActorHandle<Msg> {
  pub fn new() -> Self {
    Self {
      inner: Rc::new(RefCell::new(ActorHandleInner { actor: None })),
    }
  }

  pub fn set_actor(&mut self, actor: Rc<RefCell<dyn ActorBehavior<Msg>>>) {
    self.inner.borrow_mut().actor = Some(actor);
  }

  pub fn get_actor(&self) -> Option<Rc<RefCell<dyn ActorBehavior<Msg>>>> {
    self.inner.borrow().actor.clone()
  }

  pub fn to_any(self) -> ActorHandle<AnyMessage> {
    let mut b = self.inner.borrow();
    let result: Option<Rc<RefCell<dyn ActorBehavior<AnyMessage>>>> = match &b.actor {
      None => None,
      Some(actor) => Some(Rc::new(RefCell::new(AnyMessageActorWrapper::new(actor.clone())))),
    };

    ActorHandle {
      inner: Rc::new(RefCell::new(ActorHandleInner { actor: result })),
    }
  }
}
