use std::sync::{Arc, Mutex, RwLock};

use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_cell::ActorCellBehavior;

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::envelope::Envelope;
use crate::dispatch::mailbox::mailbox::Mailbox;

use crate::dispatch::mailbox::mailbox_type::{MailboxType, MailboxTypeBehavior};
use crate::dispatch::mailbox::MailboxBehavior;
use crate::dispatch::mailbox::MailboxInternal;
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::message::Message;
use crate::dispatch::system_message::{SystemMessage, SystemMessageQueueBehavior};

pub trait MessageDispatcherBehavior {
  fn create_mailbox<U: Message>(&self, actor_cell: ActorCellRef<U>, mailbox_type: MailboxType) -> Mailbox<U>;

  fn attach<U: Message>(&mut self, actor_cell: ActorCellRef<U>);
  fn detach<U: Message>(&mut self, actor_cell: ActorCellRef<U>);

  fn dispatch<U: Message>(&mut self, actor_cell: ActorCellRef<U>, message: Envelope);
  fn system_dispatch<U: Message>(&mut self, actor_cell: ActorCellRef<U>, system_message: &mut SystemMessage);
}

#[derive(Debug, Clone)]
struct MessageDispatcher {
  runtime: Arc<Runtime>,
  mailboxes: Arc<Mutex<Mailboxes>>,
}

#[derive(Debug, Clone)]
pub struct MessageDispatcherRef {
  inner: Arc<RwLock<MessageDispatcher>>,
  tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl PartialEq for MessageDispatcherRef {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner) && Arc::ptr_eq(&self.tasks, &other.tasks)
  }
}

impl MessageDispatcherRef {
  pub fn new_with_runtime_with_dead_letters(runtime: Arc<Runtime>, dead_letters: ActorRefRef<AnyMessage>) -> Self {
    Self {
      inner: Arc::new(RwLock::new(MessageDispatcher {
        runtime,
        mailboxes: Arc::new(Mutex::new(Mailboxes::new(dead_letters))),
      })),
      tasks: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn mailboxes(&self) -> Arc<Mutex<Mailboxes>> {
    let inner = self.inner.read().unwrap();
    inner.mailboxes.clone()
  }

  fn register<U: Message>(&mut self, _actor_cell: ActorCellRef<U>) {}

  fn unregister<U: Message>(&mut self, _actor_cell: ActorCellRef<U>) {}

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
    mut mailbox: Mailbox<U>,
    has_message_hint: bool,
    has_system_message_hint: bool,
  ) -> bool {
    log::debug!("register_for_execution(): start");
    let mutable_result = if mailbox.can_be_scheduled_for_panic(has_message_hint, has_system_message_hint) {
      log::debug!("register_for_execution(): mailbox.set_as_scheduled()");
      if mailbox.set_as_scheduled() {
        let task = {
          let inner = self.inner.read().unwrap();
          inner.runtime.spawn(async move {
            log::debug!("mailbox.execute(): start");
            mailbox.execute().await;
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

  pub fn suspend<U: Message>(&self, actor_cell: ActorCellRef<U>) {
    let mut mailbox = actor_cell.mailbox();
    if mailbox.dispatcher() == *self {
      mailbox.suspend();
    }
  }

  pub fn resume<U: Message>(&self, actor_cell: ActorCellRef<U>) {
    let mut mailbox = actor_cell.mailbox();
    if mailbox.dispatcher() == *self {
      mailbox.resume();
    }
  }
}

impl MessageDispatcherBehavior for MessageDispatcherRef {
  fn create_mailbox<U: Message>(&self, actor_cell: ActorCellRef<U>, mailbox_type: MailboxType) -> Mailbox<U> {
    let message_queue = Arc::new(Mutex::new(
      mailbox_type.create_message_queue(Some(actor_cell.self_ref())),
    ));
    Mailbox::new_with_message_queue(mailbox_type, message_queue)
  }

  fn attach<U: Message>(&mut self, actor_cell: ActorCellRef<U>) {
    self.register(actor_cell.clone());
    let mailbox = actor_cell.mailbox();
    self.register_for_execution(mailbox, false, true);
  }

  fn detach<U: Message>(&mut self, actor_cell: ActorCellRef<U>) {
    self.unregister(actor_cell);
  }

  fn dispatch<U: Message>(&mut self, receiver: ActorCellRef<U>, invocation: Envelope) {
    let mut mailbox = receiver.mailbox();
    mailbox.enqueue(receiver.self_ref(), invocation).unwrap();
    self.register_for_execution(mailbox, true, false);
  }

  fn system_dispatch<U: Message>(&mut self, receiver: ActorCellRef<U>, invocation: &mut SystemMessage) {
    let mut mailbox = receiver.mailbox();
    mailbox.system_enqueue(receiver.self_ref(), invocation);
    self.register_for_execution(mailbox, false, true);
  }
}
