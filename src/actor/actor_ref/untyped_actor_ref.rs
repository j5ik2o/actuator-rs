use std::sync::{Arc, Mutex};

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef, UntypedActorRef};
use crate::actor::actor_ref_provider::ActorRefProvider;
use crate::actor_system::{ActorSystem, ActorSystemArc};
use crate::kernel::any_message::AnyMessage;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct LocalActorRef {
  inner: Arc<Mutex<LocalActorRefInner>>,
}

#[derive(Debug, Clone)]
struct LocalActorRefInner {
  actor_cell: ActorCell,
}

pub type Sender = Option<LocalActorRef>;

impl LocalActorRef {
  pub fn new(system: ActorSystemArc) -> LocalActorRef {
    // LocalActorRefインスタンスを生成する
    let local_actor_ref = Self {
      inner: Arc::new(Mutex::new(LocalActorRefInner { actor_cell: ActorCell::new(system) })),
    };
    // 同じLocalRefInnerインスタンスへの参照を持つ新しいLocalActorRefインスタンスを生成する
    let cloned_local_actor_ref = local_actor_ref.clone();
    // local_actor_refはこの時点で消費されるので使えなくなる
    let local_actor_ref_arc = Arc::new(local_actor_ref);
    // ロックを獲得してLocalActorRefInnerのself_refを更新する。更新が完了するとcloned_local_actor_refとinnerのself_refが同じ値になる
    let mut local_actor_ref_inner = local_actor_ref_arc.inner.lock().unwrap();
    let weak = Arc::downgrade(&local_actor_ref_arc);

    local_actor_ref_inner
      .actor_cell
      .set_self_ref(local_actor_ref_arc.clone());
    cloned_local_actor_ref
  }
}

impl ActorRef for LocalActorRef {
  fn name(&self) -> &str {
    self.path().name()
  }

  fn path(&self) -> &ActorPath {
    todo!()
    //     self.actor_cell.path()
  }
}

impl UntypedActorRef for LocalActorRef {
  fn tell(self: Arc<Self>, _msg: AnyMessage, _sender: Sender) {
    todo!()
    // self
    //   .actor_cell
    //   .mailbox()
    //   .try_enqueue_any(msg, sender)
    //   .unwrap();
  }
}

impl InternalActorRef for LocalActorRef {
  fn provider(&self) -> Arc<dyn ActorRefProvider> {
    todo!()
  }

  fn start(&self) {
    todo!()
  }

  fn resume(&self) {
    todo!()
  }

  fn suspend(&self) {
    todo!()
  }

  fn stop(&self) {
    todo!()
  }

  fn tell_for_system(&self, _message: SystemMessage) {
    todo!()
  }

  fn parent(&self) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn get_child(&self, _name: Vec<String>) -> Arc<dyn InternalActorRef> {
    todo!()
  }

  fn is_local(&self) -> bool {
    true
  }

  fn is_terminated(&self) -> bool {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use crate::actor_system::LocalActorSystem;
  use crate::actor::actor_ref::untyped_actor_ref::LocalActorRef;
  use std::sync::Arc;

  #[test]
  fn test_new() {
    let system = Arc::new(LocalActorSystem::default());
    let actor_ref = LocalActorRef::new(system);
    println!("{:?}", actor_ref);
  }
}