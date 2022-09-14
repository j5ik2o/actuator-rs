use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::dead_letters_ref::DeadLettersRef;
use crate::actor::actor_ref::{ActorRefBehavior, UntypedActorRefBehavior};
use crate::dispatch::any_message::AnyMessage;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DeadLettersRefRef {
  inner: Rc<RefCell<DeadLettersRef>>,
}

impl PartialEq for DeadLettersRefRef {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.inner, &other.inner)
  }
}

impl DeadLettersRefRef {
  pub fn new(path: ActorPath) -> Self {
    Self {
      inner: Rc::new(RefCell::new(DeadLettersRef::new(path))),
    }
  }
}

impl UntypedActorRefBehavior for DeadLettersRefRef {
  fn tell_any(&mut self, msg: AnyMessage) {
    self.tell(msg);
  }
}

impl ActorRefBehavior for DeadLettersRefRef {
  type M = AnyMessage;

  fn path(&self) -> ActorPath {
    let inner = self.inner.borrow();
    inner.path()
  }

  fn tell(&mut self, msg: Self::M) {
    let mut inner = self.inner.borrow_mut();
    inner.tell(msg);
  }
}
