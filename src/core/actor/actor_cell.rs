use std::cell::RefCell;
use std::fmt::Debug;

use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use rand::{thread_rng, RngCore};
use tokio::runtime::Runtime;

use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_context::ActorContext;
use crate::core::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior, AnyActorRef, AnyActorRefBehavior};

use crate::core::actor::children_refs::ChildrenRefs;
use crate::core::actor::props::{AnyProps, Props};
use crate::core::actor::{ActorError, ActorMutableBehavior, AnyMessageActorWrapper};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::dispatcher::{Dispatcher, DispatcherBehavior};
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::mailbox::dead_letter_mailbox::DeadLetterMailbox;
use crate::core::dispatch::mailbox::mailbox::{Mailbox, MailboxSender};
use crate::core::dispatch::mailbox::mailbox_type::MailboxType;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::system_message::SystemMessage;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::SystemMessageQueueWriterBehavior;

use crate::infrastructure::logging_mutex::LoggingMutex;

use crate::mutex_lock_with_log;
use tokio::sync::oneshot;

pub const UNDEFINED_UID: u32 = 0;

pub fn new_uid() -> u32 {
  let uid = thread_rng().next_u32();
  if uid == UNDEFINED_UID {
    new_uid()
  } else {
    uid
  }
}

pub fn split_name_and_uid(name: &str) -> (&str, u32) {
  let i = name.chars().position(|c| c == '#');
  match i {
    None => (name, UNDEFINED_UID),
    Some(n) => {
      let h = &name[..n];
      let t = &name[n + 1..];
      let nn = t.parse::<u32>().unwrap();
      (h, nn)
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutoReceivedMessage {
  PoisonPill,
  Terminated(ActorRef<AnyMessage>),
}

#[derive(Debug, Clone)]
struct ActorCellInner<Msg: Message> {
  path: ActorPath,
  parent_ref: Option<AnyActorRef>,
  dispatcher: Dispatcher,
  mailbox: Option<Mailbox<Msg>>,
  dead_letter_mailbox: Option<DeadLetterMailbox>,
  mailbox_sender: Option<MailboxSender<Msg>>,
  props: Rc<dyn Props<Msg>>,
  actor: Option<Rc<RefCell<dyn ActorMutableBehavior<Msg>>>>,
  children: ChildrenRefs,
  current_message: Rc<RefCell<Option<Envelope>>>,
}

// impl<Msg: Message> Drop for ActorCellInner<Msg> {
//   fn drop(&mut self) {
//     if self.actor.is_some() {
//       log::error!("=======> ActorCellInner({}) dropped before actor", self.path);
//     }
//     log::debug!("=======> Dropping ActorCellInner({})", self.path);
//   }
// }
//
#[derive(Debug, Clone)]
pub struct ActorCell<Msg: Message> {
  initialized: Arc<AtomicBool>,
  inner: Arc<LoggingMutex<ActorCellInner<Msg>>>,
  path: ActorPath,
  terminated_rx: Arc<Mutex<Option<oneshot::Receiver<()>>>>,
  tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

// impl<Msg: Message> Drop for ActorCell<Msg> {
//   fn drop(&mut self) {
//     let sc = Arc::strong_count(&self.inner);
//     if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
//       log::error!(
//         "******> ActorCell({}) dropped before initialized, sc = {}",
//         self.path,
//         sc
//       );
//     }
//     log::debug!("******> Dropping ActorCell({}), sc = {}", self.path, sc);
//   }
// }

// impl<Msg: Message> Debug for ActorCell<Msg> {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
//       panic!("ActorCell not initialized");
//     }
//     write!(f, "ActorCell({:?})", self.inner)
//   }
// }

unsafe impl<Msg: Message> Send for ActorCell<Msg> {}
unsafe impl<Msg: Message> Sync for ActorCell<Msg> {}

impl<Msg: Message> PartialEq for ActorCell<Msg> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
    // let l = mutex_lock_with_log!(self.inner, "eq");
    // let r = mutex_lock_with_log!(self.inner, "eq");
    // l.mailbox == r.mailbox && l.current_message == r.current_message
  }
}

impl<Msg: Message> ActorCell<Msg> {
  pub fn new(
    dispatcher: Dispatcher,
    path: ActorPath,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<AnyActorRef>,
  ) -> Self {
    let (tx, rx) = oneshot::channel();
    ActorCell {
      terminated_rx: Arc::new(Mutex::new(Some(rx))),
      tx: Arc::new(Mutex::new(Some(tx))),
      path: path.clone(),
      initialized: Arc::new(AtomicBool::new(false)),
      inner: Arc::new(LoggingMutex::new(
        &format!("ActorCell#inner: {}", path.to_string()),
        ActorCellInner {
          path,
          parent_ref,
          dispatcher: dispatcher.clone(),
          mailbox: None,
          mailbox_sender: None,
          dead_letter_mailbox: None,
          props,
          actor: None,
          children: ChildrenRefs::new(),
          current_message: Rc::new(RefCell::new(None)),
        },
      )),
    }
  }

  pub fn initialize(
    &mut self,
    self_ref: ActorRef<Msg>,
    mailbox_type: MailboxType,
    dead_letter_mailbox: DeadLetterMailbox,
    send_supervise: bool,
  ) {
    if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell already initialized");
    }
    {
      let mut inner = mutex_lock_with_log!(self.inner, "initialize");
      let mailbox = inner
        .dispatcher
        .create_mailbox(Some(self_ref.clone()), mailbox_type.clone());
      inner.mailbox = Some(mailbox.clone());
      inner.mailbox_sender = Some(mailbox.sender());
      inner.dead_letter_mailbox = Some(dead_letter_mailbox);
      inner.mailbox.as_mut().unwrap().sender().system_enqueue(
        self_ref.clone(),
        &mut SystemMessageEntry::new(SystemMessage::of_create()),
      );
      self.initialized.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if send_supervise {
      let self_ref_any = self_ref.to_any(false);
      let mut parent_ref = {
        let inner = mutex_lock_with_log!(self.inner, "initialize");
        inner.parent_ref.clone()
      };
      if let Some(parent_ref) = &mut parent_ref {
        parent_ref.send_system_message(&mut SystemMessageEntry::new(SystemMessage::of_supervise(
          self_ref_any,
          true,
        )));
      }
    }
  }

  pub fn dead_letter_mailbox(&self) -> DeadLetterMailbox {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let inner = mutex_lock_with_log!(self.inner, "dead_letter_mailbox");
    inner.dead_letter_mailbox.as_ref().unwrap().clone()
  }

  fn check_actor<U: Message>(
    validate_actor: bool,
    actor: Option<Rc<RefCell<dyn ActorMutableBehavior<U>>>>,
  ) -> Option<Rc<RefCell<dyn ActorMutableBehavior<U>>>> {
    match &actor {
      None => {
        if validate_actor {
          panic!("ActorCell not initialized")
        } else {
          actor
        }
      }
      Some(_) => actor,
    }
  }

  pub fn to_any(self, validate_actor: bool) -> ActorCell<AnyMessage> {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let rc: fn(Rc<RefCell<dyn ActorMutableBehavior<Msg>>>) -> Rc<RefCell<dyn ActorMutableBehavior<AnyMessage>>> =
      |e: Rc<RefCell<dyn ActorMutableBehavior<Msg>>>| Rc::new(RefCell::new(AnyMessageActorWrapper::new(e.clone())));
    let inner = mutex_lock_with_log!(self.inner, "to_any");
    ActorCell {
      terminated_rx: self.terminated_rx.clone(),
      tx: self.tx.clone(),
      path: inner.path.clone(),
      initialized: self.initialized.clone(),
      inner: Arc::new(LoggingMutex::new(
        &format!("ActorCell#inner: {}", inner.path.to_string()),
        ActorCellInner {
          path: inner.path.clone(),
          parent_ref: inner.parent_ref.clone(),
          dispatcher: inner.dispatcher.clone(),
          mailbox: inner.mailbox.clone().map(Mailbox::to_any),
          dead_letter_mailbox: inner.dead_letter_mailbox.clone(),
          mailbox_sender: inner.mailbox_sender.clone().map(MailboxSender::to_any),
          props: inner.props.to_any(),
          actor: Self::check_actor(validate_actor, inner.actor.clone().map(rc)),
          children: inner.children.clone(),
          current_message: inner.current_message.clone(),
        },
      )),
    }
  }

  pub fn send_message(&mut self, self_ref: ActorRef<Msg>, msg: Msg) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "send_message");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    let envelope = Envelope::new(msg);
    dispatcher.dispatch(ctx, envelope);
  }

  pub fn send_system_message(&mut self, self_ref: ActorRef<Msg>, msg: &mut SystemMessageEntry) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "send_message");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.system_dispatch(ctx, msg);
  }

  pub(crate) fn new_child_actor<U: Message>(
    &mut self,
    self_ref: ActorRef<Msg>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let actor_path = ActorPath::of_child(self_ref.path(), name, 0);
    let dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "new_child_actor");
      inner.dispatcher.clone()
    };
    let mut child_actor_cell = ActorCell::new(
      dispatcher.clone(),
      actor_path.clone(),
      props,
      Some(self_ref.to_any(true)),
    );
    let actor_ref = ActorRef::of_local(child_actor_cell.clone(), actor_path);
    child_actor_cell.initialize(
      actor_ref.clone(),
      MailboxType::Unbounded,
      self.dead_letter_mailbox(),
      true,
    );
    actor_ref
  }

  pub fn actor_of<U: Message>(&mut self, self_ref: ActorRef<Msg>, props: Rc<dyn Props<U>>) -> ActorRef<U> {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let mut children = {
      let inner = mutex_lock_with_log!(self.inner, "actor_of");
      inner.children.clone()
    };
    children.actor_of(self.clone().to_any(true), self_ref.to_any(true), props)
  }

  pub fn actor_with_name_of<U: Message>(
    &mut self,
    self_ref: ActorRef<Msg>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let mut children = {
      let inner = mutex_lock_with_log!(self.inner, "actor_with_name_of");
      inner.children.clone()
    };
    let result = children.actor_with_name_of(self.clone().to_any(true), self_ref.to_any(true), props, name);
    result
  }

  pub(crate) fn start(&mut self, self_ref: ActorRef<Msg>) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    if self.exists_actor() {
      panic!("ActorCell already started")
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "start");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref.clone());
    dispatcher.attach(ctx);
  }

  fn exists_actor(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "exsits_actor");
    inner.actor.is_some()
  }

  pub(crate) fn stop(&mut self, self_ref: ActorRef<Msg>) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "stop");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref.clone());
    dispatcher.system_dispatch(ctx, &mut SystemMessageEntry::new(SystemMessage::of_terminate()));
  }

  pub(crate) fn suspend(&mut self, self_ref: ActorRef<Msg>) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) && self.exists_actor() {
      panic!("ActorCell not initialized");
    }
    if !self.exists_actor() {
      panic!("ActorCell not exists actor");
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "suspend");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.system_dispatch(ctx, &mut SystemMessageEntry::new(SystemMessage::of_suspend()))
  }

  pub fn resume(&mut self, self_ref: ActorRef<Msg>, caused_by_failure: Option<ActorError>) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    if !self.exists_actor() {
      panic!("ActorCell not exists actor");
    }
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "resume");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.system_dispatch(
      ctx,
      &mut SystemMessageEntry::new(SystemMessage::of_resume_with_failure(caused_by_failure)),
    )
  }
}

impl ActorCell<AnyMessage> {
  pub fn to_typed<Msg: Message>(self, validate_actor: bool) -> ActorCell<Msg> {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    log::debug!("check-1");
    let any_message_actor_wrapper = {
      let inner = mutex_lock_with_log!(self.inner, "to_typed");
      if let Some(actor) = inner.actor.clone() {
        let ptr = Rc::into_raw(actor).cast::<AnyMessageActorWrapper<Msg>>();
        let rc = unsafe { Rc::from_raw(ptr) };
        Some((&*rc).clone())
      } else {
        None
      }
    };
    log::debug!("check-2");
    let props = {
      let inner = mutex_lock_with_log!(self.inner, "to_typed");
      let ptr = Rc::into_raw(inner.props.clone()).cast::<AnyProps<Msg>>();
      let rc = unsafe { Rc::from_raw(ptr) };
      (&*rc).clone()
    };
    let inner = mutex_lock_with_log!(self.inner, "to_typed");
    ActorCell {
      terminated_rx: self.terminated_rx,
      tx: self.tx,
      path: inner.path.clone(),
      initialized: self.initialized.clone(),
      inner: Arc::new(LoggingMutex::new(
        &format!("ActorCell#inner: {}", inner.path.to_string()),
        ActorCellInner {
          path: inner.path.clone(),
          parent_ref: inner.parent_ref.clone(),
          dispatcher: inner.dispatcher.clone(),
          mailbox: inner.mailbox.clone().map(Mailbox::to_typed),
          dead_letter_mailbox: inner.dead_letter_mailbox.clone(),
          mailbox_sender: inner.mailbox_sender.clone().map(MailboxSender::to_typed),
          props: props.underlying,
          actor: Self::check_actor(validate_actor, any_message_actor_wrapper.map(|e| e.actor)),
          children: inner.children.clone(),
          current_message: inner.current_message.clone(),
        },
      )),
    }
  }
}

pub trait ActorCellBehavior<Msg: Message> {
  fn mailbox(&self) -> Mailbox<Msg>;

  fn mailbox_sender(&self) -> MailboxSender<Msg>;

  fn invoke(&mut self, self_ref: ActorRef<Msg>, msg: &Envelope);

  fn system_invoke(&mut self, self_ref: ActorRef<Msg>, msg: &SystemMessage);
}

impl<Msg: Message> ActorCellBehavior<Msg> for ActorCell<Msg> {
  fn mailbox(&self) -> Mailbox<Msg> {
    let inner = mutex_lock_with_log!(self.inner, "mailbox");
    inner.mailbox.clone().unwrap()
  }

  fn mailbox_sender(&self) -> MailboxSender<Msg> {
    let inner = mutex_lock_with_log!(self.inner, "mailbox_sender");
    let result = inner.mailbox_sender.as_ref().unwrap().clone();
    result
  }

  fn invoke(&mut self, self_ref: ActorRef<Msg>, msg: &Envelope) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!(
        "ActorCell not initialized: path = {}, msg = {:?}",
        self_ref.path(),
        msg.clone().typed_message::<Msg>().unwrap()
      );
    }
    // if !self.exists_actor() {
    //   panic!(
    //     "ActorCell not exists actor: path = {}, msg = {:?}",
    //     self_ref.path(),
    //     msg.clone().typed_message::<Msg>().unwrap()
    //   );
    // }
    {
      let inner = mutex_lock_with_log!(self.inner, "invoke");
      let mut current_message = inner.current_message.borrow_mut();
      *current_message = Some(msg.clone());
    }
    let ctx = ActorContext::new(self.clone(), self_ref.clone());
    let mut inner = mutex_lock_with_log!(self.inner, "invoke").clone();
    let mut actor = inner.actor.as_mut().unwrap().borrow_mut();

    let auto_received_message = msg.clone().typed_message::<AnyMessage>();
    match auto_received_message {
      Ok(msg) => match msg.take::<AutoReceivedMessage>() {
        Ok(AutoReceivedMessage::Terminated(ar)) => {
          log::info!("start - around_child_terminated");
          actor.around_child_terminated(ctx, ar.clone()).unwrap();
          log::info!("finished - around_child_terminated");
          let is_empty = {
            let mut inner = mutex_lock_with_log!(self.inner, "invoke");
            inner.children.un_reserve_child(ar.path().name());
            inner.children.is_empty()
          };
          if is_empty {
            self.tell_terminated_to_parent(self_ref);
          }
        }
        _ => {}
      },
      Err(_) => {
        let msg = msg.clone().typed_message::<Msg>().unwrap();
        actor.around_receive(ctx, msg).unwrap();
      }
    }

    {
      let inner = mutex_lock_with_log!(self.inner, "invoke");
      let mut cm = inner.current_message.borrow_mut();
      *cm = None;
    }
  }

  fn system_invoke(&mut self, self_ref: ActorRef<Msg>, msg: &SystemMessage) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      panic!("ActorCell not initialized");
    }
    match msg {
      SystemMessage::Create { failure: _ } => {
        let mut actor_rc = {
          let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
          inner.actor = Some(inner.props.new_actor());
          inner.actor.clone()
        };
        let ctx = ActorContext::new(self.clone(), self_ref);
        let mut actor = actor_rc.as_mut().unwrap().borrow_mut();
        actor.around_pre_start(ctx).unwrap();
      }
      SystemMessage::Terminate => {
        let is_empty;
        {
          let inner = mutex_lock_with_log!(self.inner, "system_invoke");
          is_empty = inner.children.is_empty();
          if inner.children.non_empty() {
            inner.children.stop_all_children();
          }
        }
        {
          let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
          match inner.actor {
            Some(ref mut actor) => {
              let ctx = ActorContext::new(self.clone(), self_ref.clone());
              let mut actor_ref_mut = actor.borrow_mut();
              actor_ref_mut.around_post_stop(ctx).unwrap();
            }
            None => {
              // log::warn!("system_invoke: actor({}) is None", self_ref.path());
            }
          }
        }
        if is_empty {
          self.tell_terminated_to_parent(self_ref);
        }
        {
          let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
          inner.children.clear();
          let parent_ref = inner.parent_ref.take();
          drop(parent_ref);
          let actor = inner.actor.take();
          drop(actor);
        }
      }
      _ => {}
    }
  }
}

impl<Msg: Message> ActorCell<Msg> {
  pub fn when_terminate(&self) {
    let runner = Runtime::new().unwrap();
    let mut rx_g = self.terminated_rx.lock().unwrap();
    let rx = rx_g.take().unwrap();
    runner.block_on(async move {
      match rx.await {
        Ok(()) => {
          log::info!("when_terminate: terminated");
        }
        Err(error) => {
          log::error!("when_terminate: error = {:?}", error);
        }
      }
    });
  }

  fn tell_terminated_to_parent(&mut self, self_ref: ActorRef<Msg>) {
    let mut parent_ref_opt = {
      let inner = mutex_lock_with_log!(self.inner, "system_invoke");
      inner.parent_ref.clone()
    };
    if let Some(parent_ref) = &mut parent_ref_opt {
      let terminated = AutoReceivedMessage::Terminated(self_ref.clone().to_any(false));
      let msg = AnyMessage::new(terminated);
      parent_ref.tell_any(msg);
    } else {
      let mut tx_g = self.tx.lock().unwrap();
      let terminated_tx = tx_g.take().unwrap();
      terminated_tx.send(()).unwrap()
    }
  }
}
