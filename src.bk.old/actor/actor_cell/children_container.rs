mod empty_container;
pub mod normal_container;
pub mod terminating_container;

use crate::actor::actor::ActorError;
use crate::actor::actor_cell::children_container::empty_container::EmptyContainerRef;
use crate::actor::actor_cell::children_container::normal_container::NormalContainerRef;
use crate::actor::actor_cell::children_container::terminating_container::TerminatingContainerRef;

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;

use crate::actor::child_state::ChildState;
use crate::dispatch::message::Message;

pub trait ChildrenContainerBehavior<Msg: Message> {
  fn add(&mut self, name: &str, stats: ChildState<Msg>) -> ChildrenContainer<Msg>;
  fn remove(&mut self, child: ActorRefRef<Msg>) -> ChildrenContainer<Msg>;

  fn get_by_name(&self, name: &str) -> Option<ChildState<Msg>>;
  fn get_by_ref(&self, actor: ActorRefRef<Msg>) -> Option<ChildState<Msg>>;

  fn children(&self) -> Vec<ActorRefRef<Msg>>;
  fn stats(&self) -> Vec<ChildState<Msg>>;

  fn shall_die(&mut self, actor: ActorRefRef<Msg>) -> ChildrenContainer<Msg>;

  fn reserve(&mut self, name: &str) -> ChildrenContainer<Msg>;
  fn unreserve(&mut self, name: &str) -> ChildrenContainer<Msg>;

  fn is_terminating(&self) -> bool {
    false
  }

  fn is_normal(&self) -> bool {
    true
  }
}

#[derive(Debug, Clone)]
pub enum ChildrenContainer<Msg: Message> {
  Normal(NormalContainerRef<Msg>),
  Empty(EmptyContainerRef<Msg>),
  Terminating(TerminatingContainerRef<Msg>),
  Terminated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuspendReason {
  UserRequest,
  Recreation { cause: ActorError },
  Creation,
  Termination,
}

impl<Msg: Message> PartialEq for ChildrenContainer<Msg> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ChildrenContainer::Normal(l), ChildrenContainer::Normal(r)) => l == r,
      (ChildrenContainer::Empty(l), ChildrenContainer::Empty(r)) => l == r,
      (ChildrenContainer::Terminating(l), ChildrenContainer::Terminating(r)) => l == r,
      (ChildrenContainer::Terminated, ChildrenContainer::Terminated) => true,
      _ => false,
    }
  }
}

impl<Msg: Message> ChildrenContainer<Msg> {
  pub fn as_terminating(&self) -> Option<&TerminatingContainerRef<Msg>> {
    match self {
      ChildrenContainer::Terminating(inner) => Some(inner),
      _ => None,
    }
  }

  pub fn of_empty() -> Self {
    ChildrenContainer::Empty(EmptyContainerRef::new())
  }

  pub fn of_normal() -> Self {
    ChildrenContainer::Normal(NormalContainerRef::new())
  }

  pub fn deep_copy(&self) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => ChildrenContainer::Normal(inner.deep_copy()),
      ChildrenContainer::Empty(inner) => ChildrenContainer::Empty(inner.deep_copy()),
      ChildrenContainer::Terminating(inner) => ChildrenContainer::Terminating(inner.deep_copy()),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }
}

impl<Msg: Message> ChildrenContainerBehavior<Msg> for ChildrenContainer<Msg> {
  fn add(&mut self, name: &str, stats: ChildState<Msg>) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => inner.add(name, stats),
      ChildrenContainer::Empty(inner) => inner.add(name, stats),
      ChildrenContainer::Terminating(inner) => inner.add(name, stats),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn remove(&mut self, child: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => inner.remove(child),
      ChildrenContainer::Empty(inner) => inner.remove(child),
      ChildrenContainer::Terminating(inner) => inner.remove(child),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState<Msg>> {
    match self {
      ChildrenContainer::Normal(inner) => inner.get_by_name(name),
      ChildrenContainer::Empty(inner) => inner.get_by_name(name),
      ChildrenContainer::Terminating(inner) => inner.get_by_name(name),
      ChildrenContainer::Terminated => None,
    }
  }

  fn get_by_ref(&self, actor: ActorRefRef<Msg>) -> Option<ChildState<Msg>> {
    match self {
      ChildrenContainer::Normal(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Empty(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Terminating(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Terminated => None,
    }
  }

  fn children(&self) -> Vec<ActorRefRef<Msg>> {
    match self {
      ChildrenContainer::Normal(inner) => inner.children(),
      ChildrenContainer::Empty(inner) => inner.children(),
      ChildrenContainer::Terminating(inner) => inner.children(),
      ChildrenContainer::Terminated => vec![],
    }
  }

  fn stats(&self) -> Vec<ChildState<Msg>> {
    match self {
      ChildrenContainer::Normal(inner) => inner.stats(),
      ChildrenContainer::Empty(inner) => inner.stats(),
      ChildrenContainer::Terminating(inner) => inner.stats(),
      ChildrenContainer::Terminated => vec![],
    }
  }

  fn shall_die(&mut self, actor: ActorRefRef<Msg>) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => inner.shall_die(actor),
      ChildrenContainer::Empty(inner) => inner.shall_die(actor),
      ChildrenContainer::Terminating(inner) => inner.shall_die(actor),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => inner.reserve(name),
      ChildrenContainer::Empty(inner) => inner.reserve(name),
      ChildrenContainer::Terminating(inner) => inner.reserve(name),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn unreserve(&mut self, name: &str) -> ChildrenContainer<Msg> {
    match self {
      ChildrenContainer::Normal(inner) => inner.unreserve(name),
      ChildrenContainer::Empty(inner) => inner.unreserve(name),
      ChildrenContainer::Terminating(inner) => inner.unreserve(name),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn is_terminating(&self) -> bool {
    match self {
      ChildrenContainer::Empty(c) => c.is_terminating(),
      ChildrenContainer::Normal(c) => c.is_terminating(),
      ChildrenContainer::Terminating(c) => c.is_terminating(),
      ChildrenContainer::Terminated => true,
    }
  }

  fn is_normal(&self) -> bool {
    match self {
      ChildrenContainer::Empty(c) => c.is_normal(),
      ChildrenContainer::Normal(c) => c.is_normal(),
      ChildrenContainer::Terminating(c) => c.is_normal(),
      ChildrenContainer::Terminated => false,
    }
  }
}
