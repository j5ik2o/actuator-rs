use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::address::Address;
use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::dispatcher::Dispatcher;
use crate::core::dispatch::mailbox::mailbox_type::MailboxType;
use crate::core::dispatch::mailboxes::Mailboxes;
use crate::core::dispatch::message::Message;

use crate::core::actor::children_refs::ChildrenRefs;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

pub trait ActorSystemBehavior: Debug {
  fn address(&self) -> Address;
  fn name(&self) -> String;
  fn child(&self, child: &str) -> ActorPath;
  fn descendant(&self, names: &[&str]) -> ActorPath;
  fn start_time(&self) -> Instant;
  fn up_time(&self) -> Duration {
    self.start_time().elapsed()
  }
  // fn dead_letters(&self) -> ActorRefRef<AnyMessage>;
  // fn mailboxes(&self) -> Arc<Mutex<Mailboxes>>;
  // fn runtime(&self) -> Arc<Runtime>;
}

#[derive(Debug, Clone)]
pub struct ActorSystemInner<Msg: Message> {
  address: Address,
  name: String,
  start_time: Instant,
  runtime: Arc<Runtime>,
  root_ref: Option<ActorRef<Msg>>,
  dead_letters: Option<ActorRef<AnyMessage>>,
  dispatcher: Option<Dispatcher>,
  mailboxes: Option<Arc<Mutex<Mailboxes>>>,
  children: ChildrenRefs,
  main_props: Option<Rc<dyn Props<Msg>>>,
}

pub struct ActorSystem<Msg: Message> {
  inner: Arc<RwLock<ActorSystemInner<Msg>>>,
}

impl<Msg: Message> ActorSystem<Msg> {
  pub fn new(runtime: Runtime, address: Address, name: &str, main_props: Rc<dyn Props<Msg>>) -> Self {
    Self {
      inner: Arc::new(RwLock::new(ActorSystemInner {
        address,
        name: name.to_string(),
        start_time: Instant::now(),
        runtime: Arc::new(runtime),
        root_ref: None,
        dead_letters: None,
        dispatcher: None,
        mailboxes: None,
        children: ChildrenRefs::new(),
        main_props: Some(main_props),
      })),
    }
  }

  pub fn when_terminate(&self) {
    {
      let inner = self.inner.write().unwrap();
      let root_ref = inner.root_ref.as_ref().unwrap();
      let actor_cell_opt = root_ref.actor_cell();
      let actor_cell = actor_cell_opt.as_ref().unwrap();
      actor_cell.when_terminate();
    }
    {
      let mut inner = self.inner.write().unwrap();
      let dead_letters = inner.dead_letters.take();
      drop(dead_letters);
      let mailboxes = inner.mailboxes.take();
      drop(mailboxes);
      inner.children = ChildrenRefs::new();
      let root_ref = inner.root_ref.take();
      drop(root_ref);
      let main_props = inner.main_props.take();
      drop(main_props);
    }
  }

  pub fn initialize(&mut self) -> ActorRef<Msg> {
    let mut inner = self.inner.write().unwrap();
    let main_path = ActorPath::of_root_with_name(inner.address.clone(), &inner.name);

    let dead_letters_path = main_path.clone().with_child("dead-letters");
    let dead_letters_ref = ActorRef::of_dead_letters(dead_letters_path.clone());
    let mailboxes = Arc::new(Mutex::new(Mailboxes::new(
      MailboxType::of_unbounded(),
      dead_letters_ref.clone(),
    )));

    let dispatcher = Dispatcher::new(inner.runtime.clone(), mailboxes.clone());
    inner.dispatcher = Some(dispatcher.clone());
    inner.dead_letters = Some(dead_letters_ref.clone());
    inner.mailboxes = Some(mailboxes.clone());

    let mut main_actor_cell = ActorCell::new(
      dispatcher.clone(),
      main_path.clone(),
      inner.main_props.as_ref().unwrap().clone(),
      None,
    );
    let main_actor_ref = ActorRef::of_local(main_actor_cell.clone(), main_path.clone());
    let dead_letter_mailbox = mailboxes.lock().unwrap().dead_letter_mailbox();

    main_actor_cell.initialize(
      main_actor_ref.clone(),
      MailboxType::of_unbounded(),
      dead_letter_mailbox,
      false,
    );
    inner.root_ref = Some(main_actor_ref.clone());
    inner.root_ref.as_ref().unwrap().clone()
  }

  pub fn join(&self) {
    let inner = self.inner.read().unwrap();
    inner.dispatcher.as_ref().unwrap().join();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::core::actor::actor_context::{ActorContext, ActorContextBehavior};
  use crate::core::actor::actor_ref::ActorRefBehavior;
  use crate::core::actor::props::FunctionProps;
  use crate::core::actor::{ActorBehavior, ActorResult};
  use std::cell::RefCell;
  use std::env;
  use tokio::runtime;

  #[derive(Debug, Clone)]
  struct TestChildActor;

  impl Drop for TestChildActor {
    fn drop(&mut self) {
      log::info!("TestChildActor drop");
    }
  }

  impl ActorBehavior<String> for TestChildActor {
    fn receive(&mut self, _ctx: ActorContext<String>, msg: String) -> ActorResult<()> {
      log::info!("TestChildActor received message: {:?}", msg);
      Ok(())
    }

    fn pre_start(&mut self, _ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestChildActor pre_start");
      Ok(())
    }

    fn post_stop(&mut self, _ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestChildActor post_stop");
      Ok(())
    }

    fn child_terminated(&mut self, _ctx: ActorContext<String>, _child: ActorRef<AnyMessage>) -> ActorResult<()> {
      log::info!("TestChildActor child_terminated");
      Ok(())
    }
  }

  #[derive(Debug, Clone)]
  struct TestActor {
    counter: u32,
    child_ref: Option<ActorRef<String>>,
  }

  impl Drop for TestActor {
    fn drop(&mut self) {
      log::info!("TestActor drop");
    }
  }

  impl TestActor {
    fn new() -> Self {
      Self {
        counter: 0,
        child_ref: None,
      }
    }
  }

  impl ActorBehavior<String> for TestActor {
    fn pre_start(&mut self, mut ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestActor pre_start");
      let props = Rc::new(FunctionProps::<String>::new(|| Rc::new(RefCell::new(TestChildActor))));
      let child_ref = ctx.spawn(props, "child");
      self.child_ref = Some(child_ref);
      Ok(())
    }

    fn post_stop(&mut self, _ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestActor post_stop");
      Ok(())
    }

    fn child_terminated(&mut self, _ctx: ActorContext<String>, _child: ActorRef<AnyMessage>) -> ActorResult<()> {
      log::info!("TestActor child_terminated");
      Ok(())
    }

    fn receive(&mut self, mut ctx: ActorContext<String>, msg: String) -> ActorResult<()> {
      log::info!("TestActor received message: {:?}", msg);
      self.counter += 1;
      // let chid_msg = format!("++{}++", msg);
      // log::info!("TestActor send child message: {:?}", chid_msg);
      // self.child_ref.as_mut().unwrap().tell(chid_msg);

      if self.counter == 2 {
        ctx.stop(ctx.self_ref().clone());
      }
      Ok(())
    }
  }

  fn init_logger() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_actor_system() {
    init_logger();
    let runtime = runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let address = Address::new("tcp", "test");
    let main_props = Rc::new(FunctionProps::new(move || Rc::new(RefCell::new(TestActor::new()))));

    let mut actor_system = ActorSystem::new(runtime, address, "test", main_props.clone());

    let mut actor_system_ref = actor_system.initialize();

    actor_system_ref.tell("test-1".to_string());
    actor_system_ref.tell("test-2".to_string());

    // actor_system.when_terminate();
    actor_system.join();
  }
}
