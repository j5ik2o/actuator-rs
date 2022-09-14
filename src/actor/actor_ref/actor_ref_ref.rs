use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::actor::actor::{Actor, ActorMutableBehavior};
use crate::actor::actor_context::ActorContextRef;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref::ActorRef;
use crate::actor::actor_ref::local_actor_ref_ref::LocalActorRefRef;
use crate::actor::actor_ref::{actor_ref, ActorRefBehavior, InternalActorRefBehavior, UntypedActorRefBehavior};
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::props::{Props, SingletonProps};
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::message::Message;
use crate::dispatch::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct ActorRefRef<Msg: Message> {
  inner: Rc<RefCell<ActorRef<Msg>>>,
}

impl<Msg: Message> Eq for ActorRefRef<Msg> {}
impl<Msg: Message> Hash for ActorRefRef<Msg> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.path().hash(state);
  }
}

unsafe impl<Msg: Message> Send for ActorRefRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorRefRef<Msg> {}

impl<Msg: Message> PartialEq for ActorRefRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.inner, &other.inner)
  }
}

#[derive(Debug, Clone)]
struct ActorWrapper<Msg: Message> {
  actor_ref: ActorRefRef<Msg>,
}

impl<Msg: Message> ActorMutableBehavior for ActorWrapper<Msg> {
  type Msg = AnyMessage;

  fn receive(&mut self, _ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) {
    let typed_msg = msg.take::<Msg>().unwrap();
    self.actor_ref.tell(typed_msg);
  }

  fn pre_start(&mut self, _ctx: ActorContextRef<Self::Msg>) {
    log::debug!("ActorWrapper: pre_start");
    let mut actor_ref = self.actor_ref.as_local().unwrap().clone();
    actor_ref.send_system_message(&mut SystemMessage::of_create(None));
  }
}

fn actor_ref_ref_wrapper<U: Message>(underlying: ActorRefRef<U>) -> ActorRefRef<AnyMessage> {
  let local_actor_ref_ref = underlying.as_local().unwrap();
  let actor_system_context = local_actor_ref_ref.actor_system_context();
  let mailbox_type = local_actor_ref_ref.mailbox_type();
  let actor_path = local_actor_ref_ref.path();

  let wrapper = Actor::of_mutable(Rc::new(RefCell::new(ActorWrapper {
    actor_ref: underlying.clone(),
  })));

  let mut actor_ref_ref = ActorRefRef::of_local(actor_path);
  let props = Rc::new(SingletonProps::new(wrapper));
  actor_ref_ref.initialize(actor_system_context, mailbox_type, props, None);
  actor_ref_ref
}

pub fn of_dead_letters(path: ActorPath) -> ActorRefRef<AnyMessage> {
  ActorRefRef {
    inner: Rc::new(RefCell::new(actor_ref::of_dead_letters(path))),
  }
}

impl<Msg: Message> ActorRefRef<Msg> {
  pub fn to_any_ref_ref(&self) -> ActorRefRef<AnyMessage> {
    actor_ref_ref_wrapper(self.clone())
  }

  pub fn new(inner: ActorRef<Msg>) -> Self {
    Self {
      inner: Rc::new(RefCell::new(inner)),
    }
  }

  pub fn of_local(path: ActorPath) -> Self {
    Self {
      inner: Rc::new(RefCell::new(ActorRef::of_local(path))),
    }
  }

  pub fn initialize(
    &mut self,
    actor_system_context: ActorSystemContext,
    mailbox_type: MailboxType,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) {
    self
      .inner
      .borrow_mut()
      .initialize(actor_system_context, mailbox_type, props, parent_ref);
  }

  pub fn as_local(&self) -> Option<LocalActorRefRef<Msg>> {
    self.inner.borrow().as_local().clone()
  }

  pub fn to_untyped_actor_ref(&self) -> Rc<RefCell<dyn UntypedActorRefBehavior>> {
    self.inner.clone()
  }
}

impl<Msg: Message> UntypedActorRefBehavior for ActorRefRef<Msg> {
  fn tell_any(&mut self, msg: AnyMessage) {
    match self {
      ActorRefRef { inner } => {
        let mut inner = inner.borrow_mut();
        inner.tell_any(msg);
      }
    }
  }
}

impl<Msg: Message> ActorRefBehavior for ActorRefRef<Msg> {
  type M = Msg;

  fn path(&self) -> ActorPath {
    self.as_local().unwrap().path().clone()
  }

  fn tell(&mut self, msg: Self::M) {
    self.as_local().unwrap().tell(msg)
  }
}
