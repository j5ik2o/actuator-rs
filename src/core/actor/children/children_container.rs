use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::children::child_state::ChildState;
use crate::core::actor::children::children_container::empty_container::EmptyContainer;
use crate::core::actor::children::children_container::normal_container::NormalContainer;
use crate::core::actor::children::children_container::terminating_container::TerminatingContainer;
use crate::core::actor::ActorError;
use crate::core::dispatch::any_message::AnyMessage;

mod empty_container;
mod normal_container;
mod terminating_container;

pub trait ChildrenContainerBehavior {
  fn add(&mut self, name: &str, stats: ChildState) -> ChildrenContainer;
  fn remove(&mut self, child: ActorRef<AnyMessage>) -> ChildrenContainer;

  fn get_all_child_stats(&self) -> Vec<ChildState>;
  fn get_by_name(&self, name: &str) -> Option<ChildState>;
  fn get_by_ref(&self, actor: ActorRef<AnyMessage>) -> Option<ChildState>;

  fn children(&self) -> Vec<ActorRef<AnyMessage>>;
  fn stats(&self) -> Vec<ChildState>;
  fn stats_mut<F>(&mut self, f: F)
  where
    F: Fn(&mut ChildState);

  fn shall_die(&mut self, actor: ActorRef<AnyMessage>) -> ChildrenContainer;

  fn reserve(&mut self, name: &str) -> ChildrenContainer;
  fn un_reserve(&mut self, name: &str) -> ChildrenContainer;

  fn is_terminating(&self) -> bool {
    false
  }

  fn is_normal(&self) -> bool {
    true
  }
}

#[derive(Debug, Clone)]
pub enum ChildrenContainer {
  Normal(NormalContainer),
  Empty(EmptyContainer),
  Terminating(TerminatingContainer),
  Terminated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuspendReason {
  UserRequest,
  Recreation { cause: ActorError },
  Creation,
  Termination,
}

impl PartialEq for ChildrenContainer {
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

impl ChildrenContainer {
  pub fn as_terminating(&self) -> Option<&TerminatingContainer> {
    match self {
      ChildrenContainer::Terminating(inner) => Some(inner),
      _ => None,
    }
  }

  pub fn of_empty() -> Self {
    ChildrenContainer::Empty(EmptyContainer::new())
  }

  pub fn of_normal() -> Self {
    ChildrenContainer::Normal(NormalContainer::new())
  }

  pub fn deep_copy(&self) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => ChildrenContainer::Normal(inner.deep_copy()),
      ChildrenContainer::Empty(inner) => ChildrenContainer::Empty(inner.deep_copy()),
      ChildrenContainer::Terminating(inner) => ChildrenContainer::Terminating(inner.deep_copy()),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }
}

impl ChildrenContainerBehavior for ChildrenContainer {
  fn add(&mut self, name: &str, stats: ChildState) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => inner.add(name, stats),
      ChildrenContainer::Empty(inner) => inner.add(name, stats),
      ChildrenContainer::Terminating(inner) => inner.add(name, stats),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn remove(&mut self, child: ActorRef<AnyMessage>) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => inner.remove(child),
      ChildrenContainer::Empty(inner) => inner.remove(child),
      ChildrenContainer::Terminating(inner) => inner.remove(child),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn get_all_child_stats(&self) -> Vec<ChildState> {
    match self {
      ChildrenContainer::Normal(inner) => inner.get_all_child_stats(),
      ChildrenContainer::Empty(inner) => inner.get_all_child_stats(),
      ChildrenContainer::Terminating(inner) => inner.get_all_child_stats(),
      ChildrenContainer::Terminated => vec![],
    }
  }

  fn get_by_name(&self, name: &str) -> Option<ChildState> {
    match self {
      ChildrenContainer::Normal(inner) => inner.get_by_name(name),
      ChildrenContainer::Empty(inner) => inner.get_by_name(name),
      ChildrenContainer::Terminating(inner) => inner.get_by_name(name),
      ChildrenContainer::Terminated => None,
    }
  }

  fn get_by_ref(&self, actor: ActorRef<AnyMessage>) -> Option<ChildState> {
    match self {
      ChildrenContainer::Normal(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Empty(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Terminating(inner) => inner.get_by_ref(actor),
      ChildrenContainer::Terminated => None,
    }
  }

  fn children(&self) -> Vec<ActorRef<AnyMessage>> {
    match self {
      ChildrenContainer::Normal(inner) => inner.children(),
      ChildrenContainer::Empty(inner) => inner.children(),
      ChildrenContainer::Terminating(inner) => inner.children(),
      ChildrenContainer::Terminated => vec![],
    }
  }

  fn stats(&self) -> Vec<ChildState> {
    match self {
      ChildrenContainer::Normal(inner) => inner.stats(),
      ChildrenContainer::Empty(inner) => inner.stats(),
      ChildrenContainer::Terminating(inner) => inner.stats(),
      ChildrenContainer::Terminated => vec![],
    }
  }

  fn stats_mut<F>(&mut self, f: F)
  where
    F: Fn(&mut ChildState), {
    match self {
      ChildrenContainer::Normal(inner) => inner.stats_mut(f),
      ChildrenContainer::Empty(inner) => inner.stats_mut(f),
      ChildrenContainer::Terminating(inner) => inner.stats_mut(f),
      ChildrenContainer::Terminated => (),
    }
  }

  fn shall_die(&mut self, actor: ActorRef<AnyMessage>) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => inner.shall_die(actor),
      ChildrenContainer::Empty(inner) => inner.shall_die(actor),
      ChildrenContainer::Terminating(inner) => inner.shall_die(actor),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn reserve(&mut self, name: &str) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => inner.reserve(name),
      ChildrenContainer::Empty(inner) => inner.reserve(name),
      ChildrenContainer::Terminating(inner) => inner.reserve(name),
      ChildrenContainer::Terminated => ChildrenContainer::Terminated,
    }
  }

  fn un_reserve(&mut self, name: &str) -> ChildrenContainer {
    match self {
      ChildrenContainer::Normal(inner) => inner.un_reserve(name),
      ChildrenContainer::Empty(inner) => inner.un_reserve(name),
      ChildrenContainer::Terminating(inner) => inner.un_reserve(name),
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
