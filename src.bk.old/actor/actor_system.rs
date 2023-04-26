use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;

use crate::actor::actor::{ActorMutableBehavior, ActorResult};
use crate::actor::actor_context::ActorContextRef;
use crate::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{actor_ref_ref, ActorRefBehavior};
use crate::actor::address::Address;
use crate::actor::children::{Children, ChildrenBehavior};
use crate::actor::props::Props;
use crate::actor::scheduler::Scheduler;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::message::Message;
use crate::dispatch::message_dispatcher::MessageDispatcherRef;

pub trait ActorSystemBehavior: Debug {
  fn address(&self) -> Address;
  fn name(&self) -> String;
  fn child(&self, child: &str) -> ActorPath;
  fn descendant(&self, names: &[&str]) -> ActorPath;
  fn start_time(&self) -> Instant;
  fn up_time(&self) -> Duration {
    self.start_time().elapsed()
  }
  fn dead_letters(&self) -> ActorRefRef<AnyMessage>;
  fn mailboxes(&self) -> Arc<Mutex<Mailboxes>>;
  fn runtime(&self) -> Arc<Runtime>;
}

#[derive(Debug, Clone)]
pub struct ActorSystemContext {
  pub(crate) runtime: Arc<Runtime>,
  pub(crate) dead_letters: ActorRefRef<AnyMessage>,
  pub(crate) dispatcher_ref: MessageDispatcherRef,
  pub(crate) scheduler: Scheduler,
}

#[derive(Debug, Clone)]
pub struct ActorSystem<Msg: Message> {
  address: Address,
  name: String,
  start_time: Instant,
  runtime: Arc<Runtime>,
  actor_ref: Option<ActorRefRef<Msg>>,
  dead_letters: Option<ActorRefRef<AnyMessage>>,
  dispatcher_ref: Option<MessageDispatcherRef>,
  mailboxes: Option<Arc<Mutex<Mailboxes>>>,
  children: Children<Msg>,
  main_props: Rc<dyn Props<Msg>>,
  _phantom_data: PhantomData<Msg>,
}

#[derive(Debug, Clone)]
pub struct ActorSystemRef<Msg: Message> {
  inner: Rc<RefCell<ActorSystem<Msg>>>,
}

#[derive(Debug, Clone)]
struct Logger;

#[derive(Debug, Clone, PartialEq)]
enum LoggingCommand {
  Debug(String),
  Info(String),
  Warn(String),
  Error(String),
}

impl ActorMutableBehavior for Logger {
  type Msg = LoggingCommand;

  fn receive(&mut self, _: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()> {
    match msg {
      LoggingCommand::Debug(msg) => log::debug!("{}", msg),
      LoggingCommand::Info(msg) => log::info!("{}", msg),
      LoggingCommand::Warn(msg) => log::warn!("{}", msg),
      LoggingCommand::Error(msg) => log::error!("{}", msg),
    }
    Ok(())
  }
}

impl<Msg: Message> ActorSystemRef<Msg> {
  pub fn new(runtime: Runtime, address: Address, name: &str, main_props: Rc<dyn Props<Msg>>) -> Self {
    let actor_system = ActorSystem {
      address,
      name: name.to_owned(),
      start_time: Instant::now(),
      runtime: Arc::new(runtime),
      actor_ref: None,
      dead_letters: None,
      dispatcher_ref: None,
      mailboxes: None,
      children: Children::new(),
      main_props,
      _phantom_data: PhantomData::default(),
    };
    let actor_system_arc_mut = Rc::new(RefCell::new(actor_system));

    Self {
      inner: actor_system_arc_mut,
    }
  }

  pub fn initialize(&mut self) -> ActorRefRef<Msg> {
    let mut inner = self.inner.borrow_mut();
    let main_path = ActorPath::of_root_with_name(inner.address.clone(), inner.name.clone());
    let dead_letters_path = main_path.clone().with_child("dead-letters");
    let dead_letters_ref_ref = actor_ref_ref::of_dead_letters(dead_letters_path);
    let mailboxes = Arc::new(Mutex::new(Mailboxes::new(dead_letters_ref_ref.clone())));
    inner.dead_letters = Some(dead_letters_ref_ref.clone());
    inner.mailboxes = Some(mailboxes);

    let dispatcher_ref =
      MessageDispatcherRef::new_with_runtime_with_dead_letters(inner.runtime.clone(), dead_letters_ref_ref.clone());

    let tick = Duration::from_millis(100);
    let actor_system_context = ActorSystemContext {
      runtime: inner.runtime.clone(),
      dead_letters: dead_letters_ref_ref.clone(),
      dispatcher_ref: dispatcher_ref.clone(),
      scheduler: Scheduler::new(tick),
    };
    let mut main_actor_ref = ActorRefRef::of_local(main_path);
    main_actor_ref.initialize(
      actor_system_context,
      MailboxType::of_unbounded(),
      inner.main_props.clone(),
      None,
    );
    inner.actor_ref = Some(main_actor_ref.clone());
    inner.dispatcher_ref = Some(dispatcher_ref.clone());

    inner.actor_ref.as_ref().unwrap().clone()
  }

  pub fn actor_ref(&self) -> ActorRefRef<Msg> {
    self.inner.borrow().actor_ref.as_ref().unwrap().clone()
  }

  pub fn join(&self) {
    let inner = self.inner.borrow();
    inner.dispatcher_ref.as_ref().unwrap().join();
  }
}

impl<Msg: Message> ActorSystemBehavior for ActorSystem<Msg> {
  fn address(&self) -> Address {
    self.address.clone()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn child(&self, child: &str) -> ActorPath {
    self.children.get_child(child).map(|child| child.path()).unwrap()
  }

  fn descendant(&self, _names: &[&str]) -> ActorPath {
    todo!()
  }

  fn start_time(&self) -> Instant {
    self.start_time.clone()
  }

  fn up_time(&self) -> Duration {
    todo!()
  }

  fn dead_letters(&self) -> ActorRefRef<AnyMessage> {
    self.dead_letters.as_ref().unwrap().clone()
  }

  fn mailboxes(&self) -> Arc<Mutex<Mailboxes>> {
    self.mailboxes.as_ref().unwrap().clone()
  }

  fn runtime(&self) -> Arc<Runtime> {
    self.runtime.clone()
  }
}

impl<Msg: Message> ActorSystemBehavior for ActorSystemRef<Msg> {
  fn address(&self) -> Address {
    todo!()
  }

  fn name(&self) -> String {
    let inner = self.inner.borrow();
    inner.name()
  }

  fn child(&self, child: &str) -> ActorPath {
    let inner = self.inner.borrow();
    inner.child(child)
  }

  fn descendant(&self, _names: &[&str]) -> ActorPath {
    todo!()
  }

  fn start_time(&self) -> Instant {
    let inner = self.inner.borrow();
    inner.start_time()
  }

  fn dead_letters(&self) -> ActorRefRef<AnyMessage> {
    let inner = self.inner.borrow();
    inner.dead_letters()
  }

  fn mailboxes(&self) -> Arc<Mutex<Mailboxes>> {
    let inner = self.inner.borrow();
    inner.mailboxes()
  }

  fn runtime(&self) -> Arc<Runtime> {
    let inner = self.inner.borrow();
    inner.runtime()
  }
}

#[cfg(test)]
mod test {
  use std::cell::RefCell;
  use std::rc::Rc;

  use crate::actor::actor::{Actor, ActorMutableBehavior, ActorResult};
  use crate::actor::actor_context::{ActorContextBehavior, ActorContextRef};
  use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
  use crate::actor::actor_ref::ActorRefBehavior;
  use crate::actor::actor_system::ActorSystemRef;
  use crate::actor::address::Address;
  use crate::actor::props::{FunctionProps, SingletonProps};

  #[derive(Debug, Clone)]
  struct TestChildActor;

  impl ActorMutableBehavior for TestChildActor {
    type Msg = String;

    fn receive(&mut self, _ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()> {
      log::debug!("TestChildActor received message: {:?}", msg);
      Ok(())
    }
  }

  #[derive(Debug, Clone)]
  struct TestActor {
    child_ref: Option<ActorRefRef<String>>,
  }

  impl TestActor {
    fn new() -> Self {
      Self { child_ref: None }
    }
  }

  impl ActorMutableBehavior for TestActor {
    type Msg = String;

    fn pre_start(&mut self, mut ctx: ActorContextRef<Self::Msg>) -> ActorResult<()> {
      let props = Rc::new(FunctionProps::new(|| {
        Actor::of_mutable(Rc::new(RefCell::new(TestChildActor)))
      }));
      let child_ref = ctx.spawn(props, "child");
      self.child_ref = Some(child_ref);
      Ok(())
    }

    fn receive(&mut self, _ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()> {
      log::debug!("TestActor received message: {:?}", msg);
      self.child_ref.as_mut().unwrap().tell(format!("++{}++", msg));
      Ok(())
    }
  }

  #[test]
  fn test() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
      .worker_threads(5)
      .enable_all()
      .build()
      .unwrap();
    let address = Address::new("tcp", "test");
    let main_actor = Actor::of_mutable(Rc::new(RefCell::new(TestActor::new())));
    let main_props = Rc::new(SingletonProps::new(main_actor));
    let mut actor_system = ActorSystemRef::new(runtime, address, "test", main_props);
    let mut actor_system_ref = actor_system.initialize();
    actor_system_ref.tell("test".to_string());
    actor_system.join();
  }
}
