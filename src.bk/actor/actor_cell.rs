use crate::actor::actor::{Actor, ActorError, ActorResult};
use crate::actor::actor_context::ActorContextRef;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::children::Children;
use crate::actor::props::Props;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::dead_letter_mailbox::DeadLetterMailbox;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox::Mailbox;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::ActuatorError;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::dispatch::message::Message;
use crate::dispatch::message_dispatcher::MessageDispatcherRef;

use crate::dispatch::message_queue::MessageQueueSize;
use crate::dispatch::system_message::SystemMessage;

pub mod actor_cell_ref;
pub mod children_container;
pub mod fault_info;

pub trait ActorCellInnerBehavior<Msg: Message> {
  fn start(&mut self);
  fn suspend(&mut self);
  fn resume(&mut self, caused_by_failure: Option<ActorError>);
  fn restart(&mut self, cause: ActorError);
  fn stop(&mut self);

  fn has_messages(&self) -> bool;
  fn number_of_messages(&self) -> MessageQueueSize;

  fn send_message(&mut self, msg: Envelope);
  fn send_system_message(&mut self, msg: &mut SystemMessage);

  fn invoke(&mut self, self_ref: ActorRefRef<Msg>, msg: Envelope);
  fn system_invoke(&mut self, self_ref: ActorRefRef<Msg>, msg: &SystemMessage);

  fn reschedule_receive_timeout(&mut self, self_ref: ActorRefRef<Msg>, runtime: Arc<Runtime>, duration: Duration);
  fn check_receive_timeout(&mut self, self_ref: ActorRefRef<Msg>, runtime: Arc<Runtime>, reschedule: bool);
  fn get_receive_timeout(&self) -> Option<Duration>;
  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg);
  fn cancel_receive_timeout(&mut self);
  fn children(&self) -> Children<AnyMessage>;
  fn children_vec(&self) -> Vec<ActorRefRef<AnyMessage>>;
  fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>>;
  fn is_terminated(&self) -> bool;
  fn parent_ref(&self) -> Option<ActorRefRef<AnyMessage>>;
  fn dispatcher(&self) -> MessageDispatcherRef;
  fn mailbox(&self) -> Mailbox<Msg>;
  fn current_message(&self) -> Rc<RefCell<Option<Envelope>>>;
  fn set_current_message(&mut self, envelope: Option<Envelope>);
  fn actor_system_context(&self) -> ActorSystemContext;
  fn mailbox_type(&self) -> MailboxType;
  fn new_actor(&mut self) -> Rc<RefCell<Actor<Msg>>>;
  fn props(&self) -> Rc<dyn Props<Msg>>;
  fn actor(&self) -> Option<Rc<RefCell<Actor<Msg>>>>;
  fn clear_actor(&mut self);
  fn fault_suspend(&mut self);
  fn fault_resume(&self, self_ref: ActorRefRef<Msg>, cause: Option<ActorError>);
  fn new_actor_context(&self, self_ref: ActorRefRef<Msg>) -> ActorContextRef<Msg>;
  fn create(&mut self, self_ref: ActorRefRef<Msg>, failure: Option<ActuatorError>) -> ActorResult<()>;
  fn fault_recreate(&mut self, self_ref: ActorRefRef<Msg>, cause: ActorError);
}

pub trait ActorCellBehavior<Msg: Message> {
  fn start(&mut self);
  fn suspend(&mut self);
  fn resume(&mut self, caused_by_failure: Option<ActorError>);
  fn restart(&mut self, cause: ActorError);
  fn stop(&mut self);

  fn has_messages(&self) -> bool;
  fn number_of_messages(&self) -> MessageQueueSize;

  fn send_message(&mut self, msg: Envelope);
  fn send_system_message(&mut self, msg: &mut SystemMessage);

  fn invoke(&mut self, msg: Envelope);
  fn system_invoke(&mut self, msg: &SystemMessage);

  fn reschedule_receive_timeout(&mut self, runtime: Arc<Runtime>, duration: Duration);
  fn check_receive_timeout(&mut self, runtime: Arc<Runtime>, reschedule: bool);
  fn get_receive_timeout(&self) -> Option<Duration>;
  fn set_receive_timeout(&mut self, timeout: Duration, msg: Msg);
  fn cancel_receive_timeout(&mut self);
  fn children(&self) -> Children<AnyMessage>;
  fn children_vec(&self) -> Vec<ActorRefRef<AnyMessage>>;
  fn dead_letter_mailbox(&self) -> Arc<Mutex<DeadLetterMailbox>>;
  fn is_terminated(&self) -> bool;
  fn parent_ref(&self) -> Option<ActorRefRef<AnyMessage>>;
  fn dispatcher(&self) -> MessageDispatcherRef;
  fn mailbox(&self) -> Mailbox<Msg>;
  fn current_message(&self) -> Rc<RefCell<Option<Envelope>>>;
  fn set_current_message(&mut self, envelope: Option<Envelope>);
  fn actor_system_context(&self) -> ActorSystemContext;
  fn mailbox_type(&self) -> MailboxType;
  fn self_ref(&self) -> ActorRefRef<Msg>;
  fn new_actor(&mut self) -> Rc<RefCell<Actor<Msg>>>;
  fn props(&self) -> Rc<dyn Props<Msg>>;
  fn actor(&self) -> Option<Rc<RefCell<Actor<Msg>>>>;
  fn clear_actor(&mut self);
  fn fault_suspend(&mut self);
  fn fault_resume(&self, cause: Option<ActorError>);
  fn new_actor_context(&self) -> ActorContextRef<Msg>;
  fn create(&mut self, failure: Option<ActuatorError>) -> ActorResult<()>;
  fn fault_recreate(&mut self, cause: ActorError);
}

pub const UNDEFINED_UID: u32 = 0;

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
