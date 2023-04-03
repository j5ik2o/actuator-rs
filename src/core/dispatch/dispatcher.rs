use crate::core::actor::actor_cell::ActorCellBehavior;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::mailbox::mailbox::Mailbox;
use crate::core::dispatch::mailbox::mailbox_type::{MailboxType, MailboxTypeBehavior};
use crate::core::dispatch::message::Message;

use crate::core::dispatch::mailbox::{MailboxReaderBehavior, MailboxWriterBehavior};

use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::dispatch::mailboxes::Mailboxes;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::SystemMessageQueueWriterBehavior;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct Dispatcher {
  runtime: Arc<Runtime>,
  mailboxes: Arc<Mutex<Mailboxes>>,
  tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

unsafe impl Send for Dispatcher {}
unsafe impl Sync for Dispatcher {}

pub trait DispatcherBehavior {
  fn create_mailbox<U: Message>(&self, self_ref: Option<ActorRef<U>>, mailbox_type: MailboxType) -> Mailbox<U>;

  fn attach<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>);
  fn detach<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>);

  fn dispatch<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>, message: Envelope);
  fn system_dispatch<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>, system_message: &mut SystemMessageEntry);
}

impl Dispatcher {
  pub fn new(runtime: Arc<Runtime>, mailboxes: Arc<Mutex<Mailboxes>>) -> Self {
    Self {
      runtime,
      mailboxes,
      tasks: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn mailboxes(&self) -> Arc<Mutex<Mailboxes>> {
    self.mailboxes.clone()
  }

  fn register<U: Message>(&mut self, _actor_cell: ActorCellWithRef<U>) {}

  fn unregister<U: Message>(&mut self, _actor_cell: ActorCellWithRef<U>) {}

  pub fn join(&self) {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();

    loop {
      let task_opt = {
        let mut tasks_guard = self.tasks.lock().unwrap();
        tasks_guard.pop()
      };

      match task_opt {
        Some(task) => {
          log::debug!("runtime.block_on(task): start");
          let _ = runtime.block_on(task);
          log::debug!("runtime.block_on(task): finished");
        }
        None => break,
      }
    }
  }

  pub fn register_for_execution<U: Message>(
    &mut self,
    actor_cell: ActorCellWithRef<U>,
    has_message_hint: bool,
    has_system_message_hint: bool,
  ) -> bool {
    log::debug!("register_for_execution(): start");
    let mut mailbox = actor_cell.mailbox();
    let mutable_result = if mailbox.can_be_scheduled_for_panic(has_message_hint, has_system_message_hint) {
      log::debug!("register_for_execution(): mailbox.set_as_scheduled()");
      if mailbox.set_as_scheduled() {
        let task = {
          let cloned_self = self.clone();
          self.runtime.spawn(async move {
            log::debug!("mailbox.execute(): start");
            mailbox.execute(actor_cell, cloned_self).await;
            log::debug!("mailbox.execute(): finished");
            ()
          })
        };
        let mut tasks_guard = self.tasks.lock().unwrap();
        tasks_guard.push(task);
        true
      } else {
        false
      }
    } else {
      false
    };
    log::debug!("register_for_execution(): finished");
    mutable_result
  }
}

impl DispatcherBehavior for Dispatcher {
  fn create_mailbox<U: Message>(&self, self_ref: Option<ActorRef<U>>, mailbox_type: MailboxType) -> Mailbox<U> {
    let message_queue = mailbox_type.create_message_queue(self_ref);
    Mailbox::new_with_message_queue(mailbox_type, message_queue)
  }

  fn attach<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>) {
    self.register(actor_cell.clone());
    self.register_for_execution(actor_cell, false, true);
  }

  fn detach<U: Message>(&mut self, actor_cell: ActorCellWithRef<U>) {
    self.unregister(actor_cell);
  }

  fn dispatch<U: Message>(&mut self, receiver: ActorCellWithRef<U>, invocation: Envelope) {
    let mut mailbox_sender = receiver.actor_cell.mailbox_sender();
    mailbox_sender.enqueue(receiver.actor_ref.clone(), invocation).unwrap();
    self.register_for_execution(receiver, true, false);
  }

  fn system_dispatch<U: Message>(&mut self, receiver: ActorCellWithRef<U>, invocation: &mut SystemMessageEntry) {
    let mut mailbox_sender = receiver.actor_cell.mailbox_sender();
    mailbox_sender.system_enqueue(receiver.actor_ref.clone(), invocation);
    self.register_for_execution(receiver, false, true);
  }
}
