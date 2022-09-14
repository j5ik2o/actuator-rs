use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;

use crate::actor::actor::ActorError;
use std::fmt::Debug;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_system::ActorSystemContext;

use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::mailbox::mailbox_type::MailboxType;

use crate::dispatch::message::Message;

use crate::dispatch::system_message::SystemMessage;

pub mod actor_ref;
pub mod actor_ref_ref;
pub mod dead_letters_ref;
pub mod dead_letters_ref_ref;
pub mod local_actor_ref;
pub mod local_actor_ref_ref;

pub trait ActorRefBehavior {
  type M: Message;
  fn path(&self) -> ActorPath;
  fn tell(&mut self, msg: Self::M);
}

pub trait InternalActorRefBehavior {
  type Msg: Message;
  fn actor_cell_ref(&self) -> ActorCellRef<Self::Msg>;
  fn actor_system_context(&self) -> ActorSystemContext;
  fn mailbox_type(&self) -> MailboxType;
  fn start(&mut self);
  fn resume(&mut self, caused_by_failure: ActorError);
  fn suspend(&mut self);
  fn stop(&mut self);
  fn restart(&mut self, cause: ActorError);
  fn send_system_message(&mut self, message: &mut SystemMessage);
}

pub trait UntypedActorRefBehavior: Debug {
  fn tell_any(&mut self, msg: AnyMessage);
}
