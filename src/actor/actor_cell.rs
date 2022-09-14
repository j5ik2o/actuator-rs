use crate::actor::actor::ActorError;
use crate::dispatch::envelope::Envelope;

use crate::dispatch::message::Message;

use crate::dispatch::message_queue::MessageQueueSize;
use crate::dispatch::system_message::SystemMessage;

pub mod actor_cell_ref;
pub mod children_container;
pub mod fault_info;

pub trait ActorCellBehavior<Msg: Message> {
  fn start(&mut self);
  fn suspend(&mut self);
  fn resume(&mut self, caused_by_failure: ActorError);
  fn restart(&mut self, cause: ActorError);
  fn stop(&mut self);

  fn has_messages(&self) -> bool;
  fn number_of_messages(&self) -> MessageQueueSize;

  fn send_message(&mut self, msg: Envelope);
  fn send_system_message(&mut self, msg: &mut SystemMessage);

  fn invoke(&mut self, msg: Envelope);
  fn system_invoke(&mut self, msg: &SystemMessage);
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
