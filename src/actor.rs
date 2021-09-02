pub use extended_cell::*;

use crate::kernel::message::Message;
use crate::actor::actor_context::ActorContext;
use std::sync::Arc;

pub mod actor_cell;
pub mod actor_context;
pub mod actor_path;
pub mod actor_ref;
pub mod actor_ref_factory;
pub mod actor_ref_provider;
pub mod address;
#[cfg(test)]
mod address_test;
pub mod cell;
pub mod extended_cell;

pub trait Actor: Send + Sync + 'static {
  type Msg: Message;

  fn context(&self) -> Arc<dyn ActorContext>;

  fn receive(&self, message: Self::Msg);

  fn pre_start(&self) {}

  fn post_stop(&self) {}

  fn pre_restart(&self) {
    self.post_stop();
  }

  fn post_restart(&self) {
    self.pre_start();
  }
}
