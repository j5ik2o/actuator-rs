use std::str::FromStr;
use std::sync::Arc;

use rand::prelude::*;
use substring::Substring;

use crate::actor::actor_path::ActorPath;
use crate::kernel::any_message_sender::AnyMessageSender;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct ActorCell {
  inner: Arc<ActorCellInner>,
}

#[derive(Debug, Clone)]
struct ActorCellInner {
  path: ActorPath,
  mailbox: Arc<dyn AnyMessageSender>,
  system_mailbox: MailboxSender<SystemMessage>,
}

impl ActorCell {
  // fn new_uid() -> u32 {
  //   let mut rng = rand::thread_rng();
  //   let uid = rng.next_u32();
  //   uid
  // }

  // pub fn split_name_and_uid(name: String) -> (String, u32) {
  //   let result = name
  //     .chars()
  //     .enumerate()
  //     .find(|(_, c)| *c == '#')
  //     .map(|(idx, _)| idx);
  //   result
  //     .map(|i| {
  //       let s = name[0..1].to_string();
  //       let i = name[i..].to_string();
  //       (s, u32::from_str(&i).unwrap())
  //     })
  //     .unwrap_or((name, 0))
  // }

  pub fn new(
    path: ActorPath,
    mailbox: Arc<dyn AnyMessageSender>,
    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      inner: Arc::from(ActorCellInner {
        path,
        mailbox,
        system_mailbox,
      }),
    }
  }

  pub fn path(&self) -> &ActorPath {
    &self.inner.path
  }

  pub fn mailbox(&self) -> Arc<dyn AnyMessageSender> {
    self.inner.mailbox.clone()
  }

  pub fn mailbox_for_system(&self) -> MailboxSender<SystemMessage> {
    self.inner.system_mailbox.clone()
  }

  // pub(crate) fn kernel(&self) -> &KernelRef {
  //   self.inner.kernel.as_ref().unwrap()
  // }
}
