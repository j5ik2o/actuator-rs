use std::sync::Arc;

use super::actor_uri::ActorUri;

#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Debug)]
struct ActorCellInner {
  uri: ActorUri,
}

impl Default for ActorCell {
  fn default() -> Self {
    ActorCell::new(ActorUri::default())
  }
}

impl ActorCell {
  pub fn new(uri: ActorUri) -> Self {
    Self {
      inner: Arc::from(ActorCellInner { uri }),
    }
  }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}
