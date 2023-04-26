use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::dead_letters_ref_ref::DeadLettersRefRef;
use crate::actor::actor_ref::local_actor_ref_ref::LocalActorRefRef;
use crate::actor::actor_ref::UntypedActorRefBehavior;
use crate::actor::actor_system::ActorSystemContext;
use crate::actor::props::Props;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::message::Message;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum ActorRef<Msg: Message> {
  NoSender,
  Local(LocalActorRefRef<Msg>),
  DeadLetters(DeadLettersRefRef),
}

pub fn of_dead_letters(path: ActorPath) -> ActorRef<AnyMessage> {
  ActorRef::DeadLetters(DeadLettersRefRef::new(path))
}

impl<Msg: Message> ActorRef<Msg> {
  pub fn of_local(path: ActorPath) -> Self {
    ActorRef::Local(LocalActorRefRef::new_with(path))
  }

  pub fn initialize(
    &mut self,
    self_ref: ActorRefRef<Msg>,
    actor_system_context: ActorSystemContext,
    mailbox_type: MailboxType,
    props: Rc<dyn Props<Msg>>,
    parent_ref: Option<ActorRefRef<AnyMessage>>,
  ) {
    match self {
      ActorRef::NoSender => {}
      ActorRef::Local(local_actor_ref) => {
        local_actor_ref.initialize(self_ref, actor_system_context, mailbox_type, props, parent_ref)
      }
      ActorRef::DeadLetters(_dead_letters_ref) => {}
    }
  }

  pub fn as_local(&self) -> Option<LocalActorRefRef<Msg>> {
    match self {
      ActorRef::NoSender => None,
      ActorRef::Local(local_actor_ref) => Some(local_actor_ref.clone()),
      ActorRef::DeadLetters(..) => None,
    }
  }

  pub fn to_actor_ref_ref(self) -> ActorRefRef<Msg> {
    ActorRefRef::new(self)
  }
}

unsafe impl<Msg: Message> Send for ActorRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorRef<Msg> {}

impl<Msg: Message> PartialEq for ActorRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ActorRef::DeadLetters(l), ActorRef::DeadLetters(r)) => l == r,
      (ActorRef::NoSender, ActorRef::NoSender) => true,
      (ActorRef::Local(local_actor_ref), ActorRef::Local(other_local_actor_ref)) => {
        local_actor_ref == other_local_actor_ref
      }
      _ => false,
    }
  }
}

impl<Msg: Message> UntypedActorRefBehavior for ActorRef<Msg> {
  fn tell_any(&mut self, msg: AnyMessage) {
    match self {
      ActorRef::NoSender => {}
      ActorRef::Local(local_actor_ref) => local_actor_ref.tell_any(msg),
      ActorRef::DeadLetters(dead_letters_ref) => dead_letters_ref.tell_any(msg),
    }
  }
}
