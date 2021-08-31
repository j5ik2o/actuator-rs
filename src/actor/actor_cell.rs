use std::sync::Arc;

use super::actor_uri::ActorUri;
use crate::kernel::any_message_sender::AnyMessageSender;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Debug, Clone)]
struct ActorCellInner {
  uri: ActorUri,
  mailbox: Arc<dyn AnyMessageSender>,
  system_mailbox: MailboxSender<SystemMessage>,
}

use substring::Substring;
use std::str::FromStr;
use rand::prelude::*;
use std::borrow::Borrow;

impl ActorCell {

  pub fn new_uid() -> u32 {
    let mut rng = rand::thread_rng();
    let uid = rng.next_u32();
    uid
  }

  pub fn split_name_and_uid(name: String) -> (String, u32) {
    let result = name
      .chars()
      .enumerate()
      .find(|(_, c)| *c == '#')
      .map(|(idx, _)| idx);
    result
      .map(|i| {
        let s = name.substring(0, 1).to_string();
        let i = name.substring(i, name.len() - 1);
        (s, u32::from_str(i).unwrap())
      })
      .unwrap_or((name, 0))
  }

  pub fn new(
    uri: ActorUri,
    mailbox: Arc<dyn AnyMessageSender>,
    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      inner: Arc::from(ActorCellInner {
        uri,
        mailbox,
        system_mailbox,
      }),
    }
  }

  pub fn mailbox(&self) -> Arc<dyn AnyMessageSender> {
    self.inner.mailbox.clone()
  }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}
