use std::rc::Rc;
use std::time::Duration;

use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;

#[derive(Debug, Clone)]
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
  pub fn to_typed<Msg: Message>(self, validate_actor: bool) -> ActorContext<Msg> {
    ActorContext {
      _phantom: std::marker::PhantomData,
      actor_cell: ActorCellWithRef::new(
        self.actor_cell.actor_cell.to_typed(validate_actor),
        self.actor_cell.actor_ref.to_typed(validate_actor),
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

#[cfg(test)]
mod tests {
  use std::cell::RefCell;
  use std::env;
  use std::rc::Rc;
  use std::sync::{Arc, Mutex};

  use crate::core::actor::actor_cell::ActorCell;
  use crate::core::actor::actor_context::ActorContext;
  use crate::core::actor::actor_path::ActorPath;
  use crate::core::actor::actor_ref::ActorRef;
  use crate::core::actor::props::Props;
  use crate::core::actor::{ActorBehavior, ActorResult};
  use crate::core::dispatch::any_message::AnyMessage;
  use crate::core::dispatch::dispatcher::Dispatcher;
  use crate::core::dispatch::mailbox::mailbox_type::MailboxType;
  use crate::core::dispatch::mailboxes::Mailboxes;
  #[derive(Debug, Clone)]
  struct TestActor;
  impl ActorBehavior<String> for TestActor {
    fn receive(&mut self, ctx: ActorContext<String>, msg: String) -> ActorResult<()> {
      todo!()
    }
  }
  impl ActorBehavior<AnyMessage> for TestActor {
    fn receive(&mut self, ctx: ActorContext<AnyMessage>, msg: AnyMessage) -> ActorResult<()> {
      todo!()
    }
  }
  #[derive(Debug, Clone)]
  struct TestProps {}
  impl TestProps {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl Props<String> for TestProps {
    fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<String>>> {
      Rc::new(RefCell::new(TestActor))
    }

    // fn to_any(&self) -> Rc<dyn Props<AnyMessage>> {
    //   Rc::new(TestProps::new())
    // }
  }
  impl Props<AnyMessage> for TestProps {
    fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<AnyMessage>>> {
      Rc::new(RefCell::new(TestActor))
    }

    // fn to_any(&self) -> Rc<dyn Props<AnyMessage>> {
    //   Rc::new(TestProps::new())
    // }
  }

  fn init_logger() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
  }
  #[test]
  fn test() {
    init_logger();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mailboxes = Mailboxes::new(
      MailboxType::Unbounded,
      ActorRef::of_dead_letters(ActorPath::from_string("test://test")),
    );
    let dispatcher = Dispatcher::new(Arc::new(runtime), Arc::new(Mutex::new(mailboxes)));
    let path = ActorPath::from_string("test://test");
    let ac: ActorCell<String> = ActorCell::new(dispatcher, path.clone(), Rc::new(TestProps {}), None);
    let ar = ActorRef::of_local(ac.clone(), path);
    let actor_context = ActorContext::new(ac, ar);
  }
}
