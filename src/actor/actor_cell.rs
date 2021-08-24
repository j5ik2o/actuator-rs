use std::sync::Arc;

use crate::kernel::mailbox::AnyMessageSender;

use super::actor_uri::ActorUri;

#[derive(Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Clone)]
struct ActorCellInner {
  uri: ActorUri,
  mailbox: Arc<dyn AnyMessageSender>
}

impl ActorCell {
  pub fn new(uri: ActorUri, mailbox: Arc<dyn AnyMessageSender>) -> Self {
    Self {
      inner: Arc::from(ActorCellInner { uri, mailbox }),
    }
  }

  pub fn mailbox(&self) -> Arc<dyn AnyMessageSender> {
    self.inner.mailbox.clone()
  }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}
