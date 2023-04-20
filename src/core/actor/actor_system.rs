use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::address::Address;
use crate::core::actor::children::Children;
use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::dispatcher::Dispatcher;
use crate::core::dispatch::mailbox::mailbox_type::MailboxType;
use crate::core::dispatch::mailboxes::Mailboxes;
use crate::core::dispatch::message::Message;

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
  children: Children,
  main_props: Rc<dyn Props<Msg>>,
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
        children: Children::new(),
        main_props,
      })),
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

    let mut main_actor_cell = ActorCell::new(dispatcher.clone(), main_path.clone(), inner.main_props.clone(), None);
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
    loop {
      let inner = self.inner.read().unwrap();
      inner.dispatcher.as_ref().unwrap().join();
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::core::actor::actor_context::{ActorContext, ActorContextBehavior};
  use crate::core::actor::actor_ref::ActorRefBehavior;
  use crate::core::actor::props::FunctionProps;
  use crate::core::actor::{ActorMutableBehavior, ActorResult};
  use std::cell::RefCell;
  use std::{env, thread};
  use tokio::runtime;

  #[derive(Debug, Clone)]
  struct TestChildActor;

  impl Drop for TestChildActor {
    fn drop(&mut self) {
      log::info!("TestChildActor drop");
    }
  }

  impl ActorMutableBehavior<String> for TestChildActor {
    fn receive(&mut self, _ctx: ActorContext<String>, msg: String) -> ActorResult<()> {
      log::info!("TestChildActor received message: {:?}", msg);
      Ok(())
    }

    fn pre_start(&mut self, _ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestChildActor start");
      Ok(())
    }

    fn post_stop(&mut self, _ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestChildActor stopped");
      Ok(())
    }
  }

  #[derive(Debug, Clone)]
  struct TestActor {
    child_ref: Option<ActorRef<String>>,
  }

  impl Drop for TestActor {
    fn drop(&mut self) {
      log::debug!("TestActor drop");
    }
  }

  impl TestActor {
    fn new() -> Self {
      Self { child_ref: None }
    }
  }

  impl ActorMutableBehavior<String> for TestActor {
    fn pre_start(&mut self, mut ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestActor start");
      // let props = Rc::new(FunctionProps::<String>::new(|| Rc::new(RefCell::new(TestChildActor))));
      // let child_ref = ctx.spawn(props, "child");
      // self.child_ref = Some(child_ref);
      Ok(())
    }

    fn post_stop(&mut self, mut ctx: ActorContext<String>) -> ActorResult<()> {
      log::info!("TestActor stopped");
      Ok(())
    }

    fn receive(&mut self, mut ctx: ActorContext<String>, msg: String) -> ActorResult<()> {
      log::info!("TestActor received message: {:?}", msg);
      // ctx.stop(self.child_ref.as_ref().unwrap().clone());
      ctx.stop(ctx.self_ref().clone());
      // self.child_ref.as_mut().unwrap().tell(format!("++{}++", msg));
      Ok(())
    }
  }

  fn init_logger() {
    let _ = env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test() {
    init_logger();
    let runtime = runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let address = Address::new("tcp", "test");
    let main_actor = || Rc::new(RefCell::new(TestActor::new()));
    let main_props = Rc::new(FunctionProps::new(move || main_actor()));
    let mut actor_system = ActorSystem::new(runtime, address, "test", main_props);
    let mut actor_system_ref = actor_system.initialize();
    actor_system_ref.tell("test-1".to_string());
    // actor_system_ref.tell("test-2".to_string());
    thread::sleep(Duration::from_secs(3));
    actor_system_ref.stop();
    actor_system.join();
  }
}
