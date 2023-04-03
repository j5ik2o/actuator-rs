use crate::actor::actor_cell::children_container::{ChildrenContainer, ChildrenContainerBehavior, SuspendReason};
use crate::actor::actor_path::ActorPathBehavior;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::ActorRefBehavior;
use crate::actor::child_state::{ChildRestartStats, ChildState};
use crate::dispatch::message::Message;

pub trait ChildrenBehavior<Msg: Message> {
  fn children(&self) -> Vec<ActorRefRef<Msg>>;
  fn get_child(&self, name: &str) -> Option<ActorRefRef<Msg>>;
  fn init_child(&mut self, actor_ref: ActorRefRef<Msg>) -> Option<ChildState<Msg>>;
}

#[derive(Debug, Clone)]
pub struct Children<Msg: Message> {
  children_refs: ChildrenContainer<Msg>,
}

impl<Msg: Message> Children<Msg> {
  pub fn new() -> Self {
    Self {
      children_refs: ChildrenContainer::of_empty(),
    }
  }

  fn swap_children_refs(
    &mut self,
    old_children_refs: ChildrenContainer<Msg>,
    mut new_children_refs: ChildrenContainer<Msg>,
  ) -> bool {
    if self.children_refs == old_children_refs {
      std::mem::swap(&mut self.children_refs, &mut new_children_refs);
      true
    } else {
      false
    }
  }

  pub fn set_children_termination_reason(&mut self, reason: SuspendReason) -> bool {
    loop {
      let current_children = self.children_refs.clone();
      match current_children {
        c @ ChildrenContainer::Terminating(..) => {
          let mut cc = c.clone().as_terminating().as_mut().unwrap().deep_copy();
          cc.set_reason(reason.clone());
          if self.swap_children_refs(c.clone(), ChildrenContainer::Terminating(cc)) {
            return true;
          }
          // Continue looping on failure
        }
        _ => return false,
      }
    }
  }

  pub fn is_terminating(&self) -> bool {
    self.children_refs.is_terminating()
  }

  pub fn is_normal(&self) -> bool {
    self.children_refs.is_normal()
  }

  pub fn stats(&self) -> Vec<ChildState<Msg>> {
    self.children_refs.stats()
  }
}

impl<Msg: Message> ChildrenBehavior<Msg> for Children<Msg> {
  fn children(&self) -> Vec<ActorRefRef<Msg>> {
    self.children_refs.children().clone()
  }

  fn get_child(&self, name: &str) -> Option<ActorRefRef<Msg>> {
    match self.children_refs.get_by_name(name) {
      Some(ChildState::ChildRestartStats(stats)) => Some(stats.child.clone()),
      _ => None,
    }
  }

  fn init_child(&mut self, actor_ref: ActorRefRef<Msg>) -> Option<ChildState<Msg>> {
    let mut cc = self.children_refs.deep_copy();
    let local_actor_ref = actor_ref.as_local().unwrap();
    let path = local_actor_ref.path();
    let name = path.name();

    while let Some(child_state) = cc.get_by_name(name) {
      match child_state {
        ChildState::ChildRestartStats(_) => {
          return Some(child_state.clone());
        }
        ChildState::ChildNameReserved => {
          let crs = ChildState::<Msg>::ChildRestartStats(ChildRestartStats::new(actor_ref.clone()));
          if self.swap_children_refs(cc.clone(), cc.add(name, crs.clone())) {
            return Some(crs);
          }
        }
      }
    }
    None
  }
}

#[cfg(test)]
mod test {

  #[test]
  fn test() {
    // let mut children = Children::new();
    // let actor_ref_ref = ActorRefRef::of_local();
    // children.init_child()
  }
}
