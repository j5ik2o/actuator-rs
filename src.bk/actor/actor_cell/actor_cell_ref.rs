use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::actor::actor::{Actor, ActorError, ActorMutableBehavior, ActorResult};
use crate::actor::actor_cell::fault_info::FaultHandling;
use crate::actor::actor_cell::{ActorCellBehavior, ActorCellInnerBehavior};
use crate::actor::actor_context::ActorContextRef;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::{ActorRefBehavior, InternalActorRefBehavior};
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::child_state::ChildState;
use crate::actor::children::{Children, ChildrenBehavior};
use crate::actor::props::Props;
use crate::actor::scheduler::Cancellable;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::dead_letter_mailbox::DeadLetterMailbox;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox::Mailbox;
use crate::ActuatorError;

use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::mailbox::{MailboxBehavior, MailboxInternal};
use crate::dispatch::message::Message;
use crate::dispatch::message_dispatcher::{MessageDispatcherBehavior, MessageDispatcherRef};
use crate::dispatch::message_queue::MessageQueueSize;
use crate::dispatch::system_message::{SystemMessage, SystemMessageQueueBehavior};

#[derive(Debug, Clone)]
struct ActorCell<Msg: Message> {
  actor_system_context: ActorSystemContext,
  props: Rc<dyn Props<Msg>>,
  actor: Option<Rc<RefCell<Actor<Msg>>>>,
  current_message: Rc<RefCell<Option<Envelope>>>,
  mailbox: Option<Mailbox<Msg>>,
  children: Children<AnyMessage>,
  parent_ref: Option<ActorRefRef<AnyMessage>>,
  receive_timeout: Rc<RefCell<Option<Duration>>>,
  receive_timeout_msg: Rc<RefCell<Option<Msg>>>,
  cancellable: Rc<RefCell<Option<Cancellable>>>,
  fault_handling: Rc<RefCell<Option<FaultHandling<Msg>>>>,
}

#[derive(Debug, Clone)]
pub struct ActorCellRef<Msg: Message> {
  inner: Rc<RefCell<ActorCell<Msg>>>,
}

unsafe impl<Msg: Message> Send for ActorCellRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorCellRef<Msg> {}

impl<Msg: Message> PartialEq for ActorCellRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.inner, &other.inner)
  }
}

#[derive(Debug, Clone)]
pub struct ExtendedCellRef<Msg: Message> {
  actor_cell_ref: ActorCellRef<Msg>,
  actor_ref: ActorRefRef<Msg>,
}
unsafe impl<Msg: Message> Send for ExtendedCellRef<Msg> {}
unsafe impl<Msg: Message> Sync for ExtendedCellRef<Msg> {}
impl<Msg: Message> PartialEq for ExtendedCellRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.actor_cell_ref == other.actor_cell_ref && self.actor_ref == other.actor_ref
  }
}
impl<Msg: Message> ExtendedCellRef<Msg> {
  pub fn new_cell_with_ref(actor_cell_ref: ActorCellRef<Msg>, actor_ref: ActorRefRef<Msg>) -> Self {
    Self {
      actor_cell_ref,
      actor_ref,
    }
  }
}
impl<Msg: Message> ActorCellBehavior<Msg> for ExtendedCellRef<Msg> {
  fn start(&mut self) {
    self.actor_cell_ref.start();
  }

  fn suspend(&mut self) {
    self.actor_cell_ref.suspend();
  }

  fn resume(&mut self, caused_by_failure: Option<ActorError>) {
    self.actor_cell_ref.resume(caused_by_failure);
  }

  fn restart(&mut self, cause: ActorError) {
    self.actor_cell_ref.restart(cause);
  }

  fn stop(&mut self) {
    self.actor_cell_ref.stop();
  }

  fn has_messages(&self) -> bool {
    self.actor_cell_ref.has_messages()
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    self.actor_cell_ref.number_of_messages()
  }

  fn send_message(&mut self, msg: Envelope) {
    self.actor_cell_ref.send_message(msg);
  }

  fn send_system_message(&mut self, msg: &mut SystemMessage) {
    self.actor_cell_ref.send_system_message(msg);
  }

  fn invoke(&mut self, msg: Envelope) {
    self.actor_cell_ref.invoke(self.actor_ref.clone(), msg);
  }

  fn system_invoke(&mut self, msg: &SystemMessage) {
    self.actor_cell_ref.system_invoke(self.actor_ref.clone(), msg);
  }

  fn reschedule_receive_timeout(&mut self, runtime: Arc<Runtime>, duration: Duration) {
    self
      .actor_cell_ref
      .reschedule_receive_timeout(self.actor_ref.clone(), runtime, duration);
  }

  fn check_receive_timeout(&mut self, runtime: Arc<Runtime>, reschedule: bool) {
    todo!()
  }

  fn get_receive_timeout(&self) -> Option<Duration> {
    todo!()
  }

  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg) {
    todo!()
  }

  fn cancel_receive_timeout(&mut self) {
    todo!()
  }

  fn children(&self) -> Children<AnyMessage> {
    self.actor_cell_ref.children()
  }

  fn children_vec(&self) -> Vec<ActorRefRef<AnyMessage>> {
    self.actor_cell_ref.children_vec()
  }

  fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>> {
    self.actor_cell_ref.dead_letter_mailbox()
  }

  fn is_terminated(&self) -> bool {
    self.actor_cell_ref.is_terminated()
  }

  fn parent_ref(&self) -> Option<ActorRefRef<AnyMessage>> {
    self.actor_cell_ref.parent_ref()
  }

  fn dispatcher(&self) -> MessageDispatcherRef {
    self.actor_cell_ref.dispatcher()
  }

  fn mailbox(&self) -> Mailbox<Msg> {
    self.actor_cell_ref.mailbox()
  }

  fn current_message(&self) -> Rc<RefCell<Option<Envelope>>> {
    self.actor_cell_ref.current_message()
  }

  fn set_current_message(&mut self, envelope: Option<Envelope>) {
    self.actor_cell_ref.set_current_message(envelope)
  }

  fn actor_system_context(&self) -> ActorSystemContext {
    todo!()
  }

  fn mailbox_type(&self) -> MailboxType {
    self.actor_cell_ref.mailbox_type()
  }

  fn self_ref(&self) -> ActorRefRef<Msg> {
    self.actor_ref.clone()
  }

  fn new_actor(&mut self) -> Rc<RefCell<Actor<Msg>>> {
    self.actor_cell_ref.new_actor()
  }

  fn props(&self) -> Rc<dyn Props<Msg>> {
    self.actor_cell_ref.props()
  }

  fn actor(&self) -> Option<Rc<RefCell<Actor<Msg>>>> {
    self.actor_cell_ref.actor()
  }

  fn clear_actor(&mut self) {
    self.actor_cell_ref.clear_actor()
  }

  fn fault_suspend(&mut self) {
    todo!()
  }

  fn fault_resume(&self, cause: Option<ActorError>) {
    self.actor_cell_ref.fault_resume(self.actor_ref.clone(), cause)
  }

  fn new_actor_context(&self) -> ActorContextRef<Msg> {
    self.actor_cell_ref.new_actor_context(self.actor_ref.clone())
  }

  fn create(&mut self, failure: Option<ActuatorError>) -> ActorResult<()> {
    self.actor_cell_ref.create(self.actor_ref.clone(), failure)
  }

  fn fault_recreate(&mut self, cause: ActorError) {
    self.actor_cell_ref.fault_recreate(self.actor_ref.clone(), cause)
  }
}

impl<Msg: Message> ActorCellRef<Msg> {
  pub fn new(
    actor_system_context: ActorSystemContext,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) -> Self {
    let s = Self::new_with_inner(Rc::new(RefCell::new(ActorCell {
      actor_system_context: actor_system_context.clone(),
      props,
      actor: None,
      current_message: Rc::new(RefCell::new(None)),
      mailbox: None,
      children: Children::new(),
      parent_ref,
      receive_timeout: Rc::new(RefCell::new(None)),
      receive_timeout_msg: Rc::new(RefCell::new(None)),
      cancellable: Rc::new(RefCell::new(None)),
      fault_handling: Rc::new(RefCell::new(None)),
    })));
    {
      let inner = s.inner.borrow_mut();
      let mut fh = inner.fault_handling.borrow_mut();
      *fh = Some(FaultHandling::new(s.clone()));
    }
    s
  }

  fn new_with_inner(inner: Rc<RefCell<ActorCell<Msg>>>) -> Self {
    Self { inner }
  }

  pub fn initialize(
    &mut self,
    actor_cell_ref: ExtendedCellRef<Msg>,
    send_supervise: bool,
    mailbox_type: MailboxType,
  ) -> Self {
    {
      let inner = self.inner.borrow_mut();
      match &inner.mailbox {
        Some(mailbox) => {
          if mailbox.is_closed() {
            panic!("mailbox is closed");
          }
        }
        _ => {}
      }
    }

    let mut new_mailbox_ref = {
      let inner = self.inner.borrow();
      let mailbox = inner
        .actor_system_context
        .dispatcher_ref
        .create_mailbox(self.clone(), mailbox_type.clone());
      mailbox
    };

    self.swap_mailbox(Some(&mut new_mailbox_ref));

    let mut inner = self.inner.borrow_mut();
    let mailbox_ref = inner.mailbox.as_mut().unwrap();
    mailbox_ref.set_actor_cell(actor_cell_ref.clone());
    mailbox_ref.system_enqueue(actor_cell_ref.self_ref(), &mut SystemMessage::of_create(None));

    if send_supervise {
      if let Some(parent) = &mut inner.parent_ref {
        parent
          .as_local()
          .unwrap()
          .send_system_message(&mut SystemMessage::of_supervise(
            actor_cell_ref.self_ref().to_any_ref_ref(),
            true,
          ));
      }
    }

    self.clone()
  }

  fn swap_mailbox(&mut self, mailbox: Option<&mut Mailbox<Msg>>) {
    let mut inner = self.inner.borrow_mut();
    match (inner.mailbox.as_mut(), mailbox) {
      (Some(mf), Some(mb)) => {
        if mf == mb {
          return;
        } else {
          let mailbox_ref = inner.mailbox.as_mut().unwrap();
          std::mem::swap(&mut *mailbox_ref, mb);
        }
      }
      (Some(_), None) => {
        inner.mailbox = None;
      }
      (None, Some(mb)) => {
        inner.mailbox = Some(mb.clone());
      }
      (None, None) => {
        return;
      }
    }
  }

  pub(crate) fn new_child_actor_ref<U: Message>(
    &mut self,
    self_ref: ActorRefRef<Msg>,
    props: Rc<dyn Props<U>>,
    name: &str,
  ) -> ActorRefRef<U> {
    let inner = { self.inner.borrow().clone() };
    let actor_path = ActorPath::of_child(self_ref.path(), name.to_string(), 0);
    let parent_ref = inner.parent_ref;

    let mut actor_ref = ActorRefRef::of_local(actor_path);

    actor_ref.initialize(
      inner.actor_system_context.clone(),
      MailboxType::of_unbounded(),
      props,
      parent_ref,
    );

    actor_ref
  }

  pub(crate) fn init_child<U: Message>(&mut self, actor_ref_ref: ActorRefRef<U>) -> Option<ChildState<AnyMessage>> {
    let mut inner = self.inner.borrow_mut();
    let wrapper = actor_ref_ref.to_any_ref_ref();
    inner.children.init_child(wrapper)
  }
}

impl<Msg: Message> ActorCellInnerBehavior<Msg> for ActorCellRef<Msg> {
  fn start(&mut self) {
    log::debug!("ActorCellRef::suspend: start");
    let mut inner = self.inner.borrow_mut();
    inner.actor_system_context.dispatcher_ref.attach(self.clone());
    log::debug!("ActorCellRef::suspend: finished");
  }

  fn suspend(&mut self) {
    log::debug!("ActorCellRef::suspend: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_suspend());
    log::debug!("ActorCellRef::suspend: finished");
  }

  fn resume(&mut self, caused_by_failure: Option<ActorError>) {
    log::debug!("ActorCellRef::resume: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_resume(caused_by_failure));
    log::debug!("ActorCellRef::resume: finished");
  }

  fn restart(&mut self, cause: ActorError) {
    log::debug!("ActorCellRef::restart: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_recreate(cause));
    log::debug!("ActorCellRef::restart: finished");
  }

  fn stop(&mut self) {
    log::debug!("ActorCellRef::stop: start");
    let mut inner = self.inner.borrow_mut();
    inner
      .actor_system_context
      .dispatcher_ref
      .system_dispatch(self.clone(), &mut SystemMessage::of_terminate());
    log::debug!("ActorCellRef::stop: finished");
  }

  fn has_messages(&self) -> bool {
    log::debug!("ActorCellRef::has_messages: start");
    let inner = self.inner.borrow();
    let mailbox_guard = inner.mailbox.as_ref().unwrap();
    let result = mailbox_guard.has_messages();
    log::debug!("ActorCellRef::has_messages: finished");
    result
  }

  fn number_of_messages(&self) -> MessageQueueSize {
    log::debug!("ActorCellRef::number_of_messages: start");
    let inner = self.inner.borrow();
    let mailbox_guard = inner.mailbox.as_ref().unwrap();
    let result = mailbox_guard.number_of_messages();
    log::debug!("ActorCellRef::number_of_messages: finished");
    result
  }

  fn send_message(&mut self, msg: Envelope) {
    log::debug!("ActorCellRef::send_message: start");
    let mut dispatcher = {
      let inner = self.inner.borrow_mut();
      inner.actor_system_context.dispatcher_ref.clone()
    };
    dispatcher.dispatch(self.clone(), msg);
    log::debug!("ActorCellRef::send_message: finished");
  }

  fn send_system_message(&mut self, msg: &mut SystemMessage) {
    log::debug!("ActorCellRef::send_system_message: start");
    let mut dispatcher = {
      let inner = self.inner.borrow_mut();
      inner.actor_system_context.dispatcher_ref.clone()
    };
    dispatcher.system_dispatch(self.clone(), msg);
    log::debug!("ActorCellRef::send_system_message: finished");
  }

  fn invoke(&mut self, self_ref: ActorRefRef<Msg>, msg: Envelope) {
    log::debug!("ActorCellRef::invoke: start");
    {
      let mut inner = self.inner.borrow_mut();
      let mut cm = inner.current_message.borrow_mut();
      *cm = Some(msg.clone());
    }
    let ctx = self.new_actor_context(self_ref);
    let result = {
      let mut inner = self.inner.borrow_mut();
      let mut actor = inner.actor.as_mut().unwrap().borrow_mut();
      actor.receive(ctx, msg.typed_message().unwrap())
    };
    match result {
      Ok(_) => {}
      Err(e) => {
        let mut inner = self.inner.borrow_mut();
        inner
          .fault_handling
          .borrow_mut()
          .as_mut()
          .unwrap()
          .handle_invoke_failure(self_ref, vec![], e.clone());
      }
    }
    {
      let mut inner = self.inner.borrow_mut();
      let mut cm = inner.current_message.borrow_mut();
      *cm = None;
    }
    log::debug!("ActorCellRef::invoke: finished");
  }

  fn system_invoke(&mut self, self_ref: ActorRefRef<Msg>, msg: &SystemMessage) {
    fn init_arc<Msg: Message>(s: &mut ActorCellRef<Msg>) -> Rc<RefCell<Actor<Msg>>> {
      let mut inner = s.inner.borrow_mut();
      let new_actor = inner.props.new_actor();
      inner.actor = Some(Rc::new(RefCell::new(new_actor)));
      inner.actor.as_ref().unwrap().clone()
    }
    fn get_arc<Msg: Message>(s: &mut ActorCellRef<Msg>) -> Rc<RefCell<Actor<Msg>>> {
      let inner = s.inner.borrow();
      inner.actor.as_ref().unwrap().clone()
    }
    log::debug!("ActorCellRef::system_invoke: start");
    match msg {
      SystemMessage::Create { failure, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Creating actor: actor_path = {}", self_ref.path());
        }
        self.create(self_ref, failure.clone()).unwrap();
      }
      SystemMessage::Recreate { cause, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Recreating actor: actor_path = {}", self_ref.path());
        }
        self.fault_recreate(self_ref, cause.clone());
      }
      SystemMessage::Suspend { .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Suspending actor: actor_path = {}", self_ref.path());
        }
        self.fault_suspend();
      }
      SystemMessage::Resume { caused_by_failure, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!("Resuming actor: actor_path = {}", self_ref.path());
        }
        self.fault_resume(self_ref, caused_by_failure.clone());
      }
      SystemMessage::Terminate { .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!(
            "Terminating actor: actor_path = {}",
            self_ref.as_local().unwrap().path()
          );
        }
        let actor_rc = get_arc(self);
        let ctx = ActorContextRef::new(self.clone(), self_ref);
        let mut actor = actor_rc.borrow_mut();
        actor.post_stop(ctx);
      }
      SystemMessage::Supervise { child, .. } => {
        {
          let inner = self.inner.borrow();
          log::debug!(
            "Supervising actor: actor_path = {}",
            self_ref.as_local().unwrap().path()
          );
        }
        if !self.is_terminated() {
          match self.init_child(child.clone()) {
            Some(_) => {
              // TODO: handle_supervise
            }
            None => {}
          }
        }
      }
      _ => {}
    }
    log::debug!("ActorCellRef::system_invoke: finished");
  }

  fn reschedule_receive_timeout(&mut self, self_ref: ActorRefRef<Msg>, runtime: Arc<Runtime>, duration: Duration) {
    let inner = self.inner.borrow_mut();
    {
      let mut cancellable_ref = inner.cancellable.borrow_mut();
      if let Some(cancellable) = cancellable_ref.as_mut() {
        cancellable.cancel();
      }
    }
    let result = inner.actor_system_context.scheduler.schedule_once_to_actor_ref(
      runtime,
      duration,
      self_ref.clone(),
      inner.receive_timeout_msg.borrow().clone().unwrap(),
    );
    let mut rt = inner.receive_timeout.borrow_mut();
    *rt = Some(duration);
    let mut cancellable_ref = inner.cancellable.borrow_mut();
    *cancellable_ref = Some(result);
  }

  fn check_receive_timeout(&mut self, self_ref: ActorRefRef<Msg>, runtime: Arc<Runtime>, reschedule: bool) {
    if reschedule {
      let rt_opt = {
        let inner = self.inner.borrow_mut();
        let x = inner.receive_timeout.borrow().clone();
        x
      };
      match rt_opt {
        Some(rt) => self.reschedule_receive_timeout(self_ref, runtime, rt),
        _ => self.cancel_receive_timeout(),
      }
    }
  }

  fn get_receive_timeout(&self) -> Option<Duration> {
    let inner = self.inner.borrow();
    let rt = inner.receive_timeout.borrow_mut();
    rt.clone()
  }

  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg) {
    let inner = self.inner.borrow_mut();
    let mut rt = inner.receive_timeout.borrow_mut();
    *rt = Some(timeout);
    let mut rtm = inner.receive_timeout_msg.borrow_mut();
    *rtm = Some(msg);
  }

  fn cancel_receive_timeout(&mut self) {
    log::debug!("cancel_receive_timeout: start");
    let inner = self.inner.borrow_mut();
    let mut cancellable = inner.cancellable.borrow_mut();
    match &mut *cancellable {
      Some(c) => {
        c.cancel();
      }
      None => {}
    }
    let mut rt = inner.receive_timeout.borrow_mut();
    *rt = None;
    let mut rtm = inner.receive_timeout_msg.borrow_mut();
    *rtm = None;
    log::debug!("cancel_receive_timeout: finished");
  }

  fn children(&self) -> Children<AnyMessage> {
    let inner = self.inner.borrow();
    inner.children.clone()
  }

  fn children_vec(&self) -> Vec<ActorRefRef<AnyMessage>> {
    let inner = self.inner.borrow();
    inner.children.children().clone()
  }

  fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>> {
    let inner = self.inner.borrow();
    let mailboxes = inner.actor_system_context.dispatcher_ref.mailboxes();
    let mailboxes_guard = mailboxes.lock().unwrap();
    mailboxes_guard.dead_letter_mailbox()
  }

  fn is_terminated(&self) -> bool {
    let inner = self.inner.borrow();
    let mailbox_ref = inner.mailbox.as_ref().unwrap();
    mailbox_ref.is_closed()
  }

  fn parent_ref(&self) -> Option<ActorRefRef<AnyMessage>> {
    let inner = self.inner.borrow();
    inner.parent_ref.clone()
  }

  fn dispatcher(&self) -> MessageDispatcherRef {
    let inner = self.inner.borrow();
    inner.actor_system_context.dispatcher_ref.clone()
  }

  fn mailbox(&self) -> Mailbox<Msg> {
    let inner = self.inner.borrow();
    inner.mailbox.as_ref().unwrap().clone()
  }

  fn current_message(&self) -> Rc<RefCell<Option<Envelope>>> {
    let inner = self.inner.borrow();
    inner.current_message.clone()
  }

  fn set_current_message(&mut self, envelope: Option<Envelope>) {
    let inner = self.inner.borrow();
    let mut cm = inner.current_message.borrow_mut();
    *cm = envelope;
  }

  fn actor_system_context(&self) -> ActorSystemContext {
    let inner = self.inner.borrow();
    inner.actor_system_context.clone()
  }

  fn mailbox_type(&self) -> MailboxType {
    let inner = self.inner.borrow();
    inner.mailbox.as_ref().unwrap().mailbox_type()
  }

  // fn self_ref(&self) -> ActorRefRef<Msg> {
  //   let inner = self.inner.borrow();
  //   inner.actor_ref.clone()
  // }

  fn new_actor(&mut self) -> Rc<RefCell<Actor<Msg>>> {
    let actor_rc = {
      let inner = self.inner.borrow();
      let actor = inner.props.new_actor();
      Rc::new(RefCell::new(actor))
    };
    let mut inner = self.inner.borrow_mut();
    inner.actor = Some(actor_rc.clone());
    actor_rc
  }

  fn props(&self) -> Rc<dyn Props<Msg>> {
    let inner = self.inner.borrow();
    inner.props.clone()
  }

  fn actor(&self) -> Option<Rc<RefCell<Actor<Msg>>>> {
    let inner = self.inner.borrow();
    inner.actor.clone()
  }

  fn clear_actor(&mut self) {
    let mut inner = self.inner.borrow_mut();
    inner.actor = None;
  }

  fn fault_suspend(&mut self) {
    let fh_rc = {
      let inner = self.inner.borrow();
      inner.fault_handling.clone()
    };
    let mut fh_opt = fh_rc.borrow_mut();
    let fh = fh_opt.as_mut().unwrap();
    fh.fault_suspend()
  }

  fn fault_resume(&self, self_ref: ActorRefRef<Msg>, cause: Option<ActorError>) {
    let fh_rc = {
      let inner = self.inner.borrow();
      inner.fault_handling.clone()
    };
    let mut fh_opt = fh_rc.borrow_mut();
    let fh = fh_opt.as_mut().unwrap();
    fh.fault_resume(self_ref, cause)
  }

  fn new_actor_context(&self, self_ref: ActorRefRef<Msg>) -> ActorContextRef<Msg> {
    ActorContextRef::new(self.clone(), self_ref)
  }

  fn create(&mut self, self_ref: ActorRefRef<Msg>, failure: Option<ActuatorError>) -> ActorResult<()> {
    let fh_rc = {
      let inner = self.inner.borrow();
      inner.fault_handling.clone()
    };
    let mut fh_opt = fh_rc.borrow_mut();
    let fh = fh_opt.as_mut().unwrap();
    fh.create(self_ref, failure)
  }

  fn fault_recreate(&mut self, self_ref: ActorRefRef<Msg>, cause: ActorError) {
    let fh_rc = {
      let inner = self.inner.borrow();
      inner.fault_handling.clone()
    };
    let mut fh_opt = fh_rc.borrow_mut();
    let fh = fh_opt.as_mut().unwrap();
    fh.fault_recreate(self_ref, cause.clone())
  }
}
