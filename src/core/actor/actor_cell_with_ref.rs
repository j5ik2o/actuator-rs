use crate::core::actor::actor_cell::{ActorCell, ActorCellBehavior};
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::actor::props::Props;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::mailbox::dead_letter_mailbox::DeadLetterMailbox;
use crate::core::dispatch::mailbox::mailbox::Mailbox;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::system_message::SystemMessage;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ActorCellWithRef<Msg: Message> {
  pub actor_cell: ActorCell<Msg>,
  pub actor_ref: ActorRef<Msg>,
}

unsafe impl<Msg: Message> Send for ActorCellWithRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorCellWithRef<Msg> {}

impl ActorCellWithRef<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> ActorCellWithRef<Msg> {
    ActorCellWithRef {
      actor_cell: self.actor_cell.to_typed(),
      actor_ref: self.actor_ref.to_typed(),
    }
  }
}

impl<Msg: Message> ActorCellWithRef<Msg> {
  pub fn new(actor_cell: ActorCell<Msg>, actor_ref: ActorRef<Msg>) -> Self {
    Self { actor_cell, actor_ref }
  }

  pub fn to_any(self) -> ActorCellWithRef<AnyMessage> {
    ActorCellWithRef::new(self.actor_cell.to_any(), self.actor_ref.to_any())
  }

  pub fn invoke(&mut self, msg: &Envelope) {
    self.actor_cell.invoke(self.actor_ref.clone(), msg);
  }

  pub fn system_invoke(&mut self, msg: &SystemMessage) {
    self.actor_cell.system_invoke(self.actor_ref.clone(), msg);
  }

  pub fn mailbox(&self) -> Mailbox<Msg> {
    self.actor_cell.mailbox()
  }

  pub fn dead_letter_mailbox(&self) -> DeadLetterMailbox {
    self.actor_cell.dead_letter_mailbox()
  }

  pub fn actor_of<U: Message>(&mut self, props: Rc<dyn Props<U>>) -> ActorRef<U> {
    self.actor_cell.actor_of(self.actor_ref.clone(), props)
  }

  pub fn actor_with_name_of<U: Message>(&mut self, props: Rc<dyn Props<U>>, name: &str) -> ActorRef<U> {
    log::debug!("actor_with_name_of: {}", name);
    self.actor_cell.actor_with_name_of(self.actor_ref.clone(), props, name)
  }
}
