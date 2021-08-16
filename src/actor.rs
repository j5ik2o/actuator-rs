use crate::kernel::Message;

mod actor_cell;
mod actor_path;
mod actor_ref;
mod actor_uri;
mod address;
mod context;
mod extended_cell;

pub use extended_cell::*;

pub trait Actor: Send + Sync + 'static {
  type Msg: Message;

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
