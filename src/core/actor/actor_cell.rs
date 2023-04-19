use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use rand::{thread_rng, RngCore};

use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_context::ActorContext;
use crate::core::actor::actor_path::ActorPath;
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior, AnyActorRef};
use crate::core::actor::children::Children;
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
use crate::infrastructure::logging_rw_lock::LoggingRwLock;
use crate::mutex_lock_with_log;

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

#[derive(Debug, Clone)]
struct ActorCellInner<Msg: Message> {
  parent_ref: Option<AnyActorRef>,
  dispatcher: Dispatcher,
  mailbox: Option<Mailbox<Msg>>,
  dead_letter_mailbox: Option<DeadLetterMailbox>,
  mailbox_sender: Option<MailboxSender<Msg>>,
  props: Rc<dyn Props<Msg>>,
  actor: Option<Rc<RefCell<dyn ActorMutableBehavior<Msg>>>>,
  children: Children,
  current_message: Rc<RefCell<Option<Envelope>>>,
}

#[derive(Clone)]
pub struct ActorCell<Msg: Message> {
  inner: Arc<LoggingMutex<ActorCellInner<Msg>>>,
}

impl<Msg: Message> Debug for ActorCell<Msg> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "ActorCell(***)")
  }
}

unsafe impl<Msg: Message> Send for ActorCell<Msg> {}
unsafe impl<Msg: Message> Sync for ActorCell<Msg> {}

impl<Msg: Message> PartialEq for ActorCell<Msg> {
  fn eq(&self, _other: &Self) -> bool {
    let l = mutex_lock_with_log!(self.inner, "eq");
    let r = mutex_lock_with_log!(self.inner, "eq");
    l.mailbox == r.mailbox && l.current_message == r.current_message
  }
}

impl<Msg: Message> ActorCell<Msg> {
  pub fn new(dispatcher: Dispatcher, props: Rc<dyn Props<Msg>>, parent_ref: Option<AnyActorRef>) -> Self {
    ActorCell {
      inner: Arc::new(LoggingMutex::new(
        "inner",
        ActorCellInner {
          parent_ref,
          dispatcher: dispatcher.clone(),
          mailbox: None,
          mailbox_sender: None,
          dead_letter_mailbox: None,
          props,
          actor: None,
          children: Children::new(),
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
    let mut mailbox = {
      let inner = mutex_lock_with_log!(self.inner, "initialize");
      inner
        .dispatcher
        .create_mailbox(Some(self_ref.clone()), mailbox_type.clone())
    };
    self.swap_mailbox(Some(&mut mailbox));
    {
      let mut inner = mutex_lock_with_log!(self.inner, "initialize");
      inner.mailbox_sender = Some(mailbox.sender());
      inner.dead_letter_mailbox = Some(dead_letter_mailbox);
    }
    {
      let mut inner = mutex_lock_with_log!(self.inner, "initialize");
      inner.mailbox.as_mut().unwrap().sender().system_enqueue(
        self_ref.clone(),
        &mut SystemMessageEntry::new(SystemMessage::of_create()),
      );
    }

    if send_supervise {
      let mut parent_ref = {
        let inner = mutex_lock_with_log!(self.inner, "initialize");
        inner.parent_ref.clone()
      };
      if let Some(parent_ref) = &mut parent_ref {
        parent_ref.send_system_message(&mut SystemMessageEntry::new(SystemMessage::of_supervise(
          self_ref.to_any(),
          true,
        )));
      }
    }
  }

  pub fn dead_letter_mailbox(&self) -> DeadLetterMailbox {
    let inner = mutex_lock_with_log!(self.inner, "dead_letter_mailbox");
    inner.dead_letter_mailbox.as_ref().unwrap().clone()
  }

  fn swap_mailbox(&mut self, mailbox: Option<&mut Mailbox<Msg>>) {
    let mut self_mailbox = {
      let inner = mutex_lock_with_log!(self.inner, "swap_mailbox");
      inner.mailbox.clone()
    };
    match (self_mailbox.as_mut(), mailbox) {
      (Some(mf), Some(mb)) => {
        if mf == mb {
          return;
        } else {
          let m = self_mailbox.as_mut().unwrap();
          std::mem::swap(m, mb);
        }
      }
      (Some(_), None) => {
        let mut inner = mutex_lock_with_log!(self.inner, "swap_mailbox");
        inner.mailbox = None;
      }
      (None, Some(mb)) => {
        let mut inner = mutex_lock_with_log!(self.inner, "swap_mailbox");
        inner.mailbox = Some(mb.clone());
      }
      (None, None) => {
        return;
      }
    }
  }

  pub fn to_any(self) -> ActorCell<AnyMessage> {
    let rc: fn(Rc<RefCell<dyn ActorMutableBehavior<Msg>>>) -> Rc<RefCell<dyn ActorMutableBehavior<AnyMessage>>> =
      |e: Rc<RefCell<dyn ActorMutableBehavior<Msg>>>| Rc::new(RefCell::new(AnyMessageActorWrapper::new(e.clone())));
    let inner = {
      let inner = mutex_lock_with_log!(self.inner, "to_any");
      inner.clone()
    };
    ActorCell {
      inner: Arc::new(LoggingMutex::new(
        "inner",
        ActorCellInner {
          parent_ref: inner.parent_ref.clone(),
          dispatcher: inner.dispatcher.clone(),
          mailbox: inner.mailbox.clone().map(Mailbox::to_any),
          dead_letter_mailbox: inner.dead_letter_mailbox.clone(),
          mailbox_sender: inner.mailbox_sender.clone().map(MailboxSender::to_any),
          props: inner.props.to_any(),
          actor: inner.actor.clone().map(rc),
          children: inner.children.clone(),
          current_message: inner.current_message.clone(),
        },
      )),
    }
  }

  pub fn send_message(&mut self, self_ref: ActorRef<Msg>, msg: Msg) {
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "send_message");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    let envelope = Envelope::new(msg);
    dispatcher.dispatch(ctx, envelope);
  }

  pub fn send_system_message(&mut self, self_ref: ActorRef<Msg>, msg: &mut SystemMessageEntry) {
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
    let actor_path = ActorPath::of_child(self_ref.path(), name, new_uid());
    let dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "new_child_actor");
      inner.dispatcher.clone()
    };
    let mut child_actor_cell = ActorCell::new(dispatcher.clone(), props, Some(self_ref.to_any()));
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
    let mut children = {
      let inner = mutex_lock_with_log!(self.inner, "actor_of");
      inner.children.clone()
    };
    children.actor_of(self.clone().to_any(), self_ref.to_any(), props)
  }

  pub fn actor_with_name_of<U: Message>(
    &mut self,
    self_ref: ActorRef<Msg>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRef<U> {
    log::debug!("actor_with_name_of: start: name = {}", name);
    let mut children = {
      log::debug!("actor_with_name_of: self.inner.lock().unwrap(): start {}", name);
      let inner = mutex_lock_with_log!(self.inner, "actor_with_name_of");
      log::debug!("actor_with_name_of: self.inner.lock().unwrap(): finished {}", name);
      inner.children.clone()
    };
    let result = children.actor_with_name_of(self.clone().to_any(), self_ref.to_any(), props, name);
    log::debug!("actor_with_name_of: finished: name = {}", name);
    result
  }

  pub fn start(&mut self, self_ref: ActorRef<Msg>) {
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "start");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.attach(ctx)
  }

  pub fn stop(&mut self, self_ref: ActorRef<Msg>) {
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "stop");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.system_dispatch(ctx, &mut SystemMessageEntry::new(SystemMessage::of_terminate()))
  }

  pub fn suspend(&mut self, self_ref: ActorRef<Msg>) {
    let mut dispatcher = {
      let inner = mutex_lock_with_log!(self.inner, "suspend");
      inner.dispatcher.clone()
    };
    let ctx = ActorCellWithRef::new(self.clone(), self_ref);
    dispatcher.system_dispatch(ctx, &mut SystemMessageEntry::new(SystemMessage::of_suspend()))
  }

  pub fn resume(&mut self, self_ref: ActorRef<Msg>, caused_by_failure: Option<ActorError>) {
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
  pub fn to_typed<Msg: Message>(self) -> ActorCell<Msg> {
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
    let props = {
      let inner = mutex_lock_with_log!(self.inner, "to_typed");
      let ptr = Rc::into_raw(inner.props.clone()).cast::<AnyProps<Msg>>();
      let rc = unsafe { Rc::from_raw(ptr) };
      (&*rc).clone()
    };
    let inner = mutex_lock_with_log!(self.inner, "to_typed");
    ActorCell {
      inner: Arc::new(LoggingMutex::new(
        "inner",
        ActorCellInner {
          parent_ref: inner.parent_ref.clone(),
          dispatcher: inner.dispatcher.clone(),
          mailbox: inner.mailbox.clone().map(Mailbox::to_typed),
          dead_letter_mailbox: inner.dead_letter_mailbox.clone(),
          mailbox_sender: inner.mailbox_sender.clone().map(MailboxSender::to_typed),
          props: props.underlying,
          actor: any_message_actor_wrapper.map(|e| e.actor),
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
    // log::debug!("mailbox_sender: start");
    let inner = mutex_lock_with_log!(self.inner, "mailbox_sender");
    let result = inner.mailbox_sender.as_ref().unwrap().clone();
    // log::debug!("mailbox_sender: finished");
    result
  }

  fn invoke(&mut self, self_ref: ActorRef<Msg>, msg: &Envelope) {
    {
      let inner = mutex_lock_with_log!(self.inner, "invoke");
      let mut current_message = inner.current_message.borrow_mut();
      *current_message = Some(msg.clone());
    }
    let ctx = ActorContext::new(self.clone(), self_ref.clone());
    let mut inner = mutex_lock_with_log!(self.inner, "invoke").clone();
    let mut actor = inner.actor.as_mut().unwrap().borrow_mut();
    actor
      .around_receive(ctx, msg.clone().typed_message::<Msg>().unwrap())
      .unwrap();
    {
      let inner = mutex_lock_with_log!(self.inner, "invoke");
      let mut cm = inner.current_message.borrow_mut();
      *cm = None;
    }
  }

  fn system_invoke(&mut self, self_ref: ActorRef<Msg>, msg: &SystemMessage) {
    log::debug!("system_invoke: start: self_ref = {}, {:?}", self_ref.path(), msg);
    match msg {
      SystemMessage::Create { failure: _ } => {
        let actor_rc = {
          let actor_rc = {
            let inner = mutex_lock_with_log!(self.inner, "system_invoke");
            inner.props.new_actor()
          };
          {
            let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
            inner.actor = Some(actor_rc.clone());
          }
          actor_rc
        };
        let ctx = ActorContext::new(self.clone(), self_ref);
        let mut actor = actor_rc.borrow_mut();
        actor.around_pre_start(ctx).unwrap();
      }
      SystemMessage::Terminate => {
        {
          let inner = mutex_lock_with_log!(self.inner, "system_invoke");
          log::debug!(
            "system_invoke: path = {}, children: {:?}",
            self_ref.path(),
            inner.children.children()
          );
          inner.children.children().iter_mut().for_each(|child| {
            child.stop();
          });
        }
        {
          let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
          inner.children.set_terminated();
        }
        {
          let ctx = ActorContext::new(self.clone(), self_ref);
          let mut inner = mutex_lock_with_log!(self.inner, "system_invoke");
          let mut actor = inner.actor.as_mut().unwrap().borrow_mut();
          actor.around_post_stop(ctx).unwrap();
        }
      }
      _ => {}
    }
    log::debug!("system_invoke: finished");
  }
}
