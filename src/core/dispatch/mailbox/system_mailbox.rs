use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::earliest_first_system_message_list::EarliestFirstSystemMessageList;
use crate::core::dispatch::system_message::latest_first_system_message_list::LatestFirstSystemMessageList;
use crate::core::dispatch::system_message::system_message::SystemMessage;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::system_message_list::SystemMessageList;
use crate::core::dispatch::system_message::{SystemMessageQueueReaderBehavior, SystemMessageQueueWriterBehavior, LNIL};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SystemMailbox<Msg: Message> {
  system_message_opt: Arc<Mutex<Option<SystemMessageEntry>>>,
  _phantom: std::marker::PhantomData<Msg>,
}

unsafe impl<Msg: Message> Send for SystemMailbox<Msg> {}
unsafe impl<Msg: Message> Sync for SystemMailbox<Msg> {}

impl<Msg: Message> PartialEq for SystemMailbox<Msg> {
  fn eq(&self, other: &Self) -> bool {
    if Arc::ptr_eq(&self.system_message_opt, &other.system_message_opt) {
      true
    } else {
      let l = self.system_message_opt.lock().unwrap();
      let r = other.system_message_opt.lock().unwrap();
      &*l == &*r
    }
  }
}

impl SystemMailbox<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> SystemMailbox<Msg> {
    SystemMailbox::<Msg> {
      system_message_opt: self.system_message_opt,
      _phantom: std::marker::PhantomData,
    }
  }
}

impl<Msg: Message> SystemMailbox<Msg> {
  pub fn new() -> Self {
    Self {
      system_message_opt: Arc::new(Mutex::new(None)),
      _phantom: std::marker::PhantomData,
    }
  }

  pub fn to_any(self) -> SystemMailbox<AnyMessage> {
    SystemMailbox {
      system_message_opt: self.system_message_opt,
      _phantom: std::marker::PhantomData,
    }
  }

  fn set_system_message_opt(&mut self, system_message_opt: Option<SystemMessageEntry>) {
    let mut system_message_opt_guard = self.system_message_opt.lock().unwrap();
    *system_message_opt_guard = system_message_opt;
  }

  pub fn sender(&self) -> SystemMailboxSender<Msg> {
    SystemMailboxSender {
      underlying: self.clone(),
      _phantom: std::marker::PhantomData,
    }
  }

  fn system_queue_get(&self) -> LatestFirstSystemMessageList {
    let system_message_opt_guard = self.system_message_opt.lock().unwrap();
    LatestFirstSystemMessageList::new(system_message_opt_guard.clone())
  }

  pub async fn process_all_system_messages<CloseF, TerminateF>(
    &mut self,
    is_closed: CloseF,
    is_terminate: TerminateF,
    mut actor_cell: ActorCellWithRef<Msg>,
  ) where
    CloseF: Fn() -> bool,
    TerminateF: Fn() -> bool, {
    let mut message_list = self.system_drain(&LNIL);
    log::debug!("message_list = {:?}", message_list);
    let mut error_msg: Option<String> = None;
    while message_list.non_empty() && !is_closed() {
      message_list = {
        let (head, tail) = message_list.head_with_tail().unwrap().clone();
        let mut msg = head.lock().unwrap();
        msg.unlink();
        actor_cell.system_invoke(&msg.message);
        if is_terminate() {
          error_msg = Some("Interrupted while processing system messages".to_owned());
        }
        tail
      };
      if message_list.is_empty() && !is_closed() {
        message_list = self.system_drain(&LNIL);
      }
    }
    let mut dead_letter_mailbox = if message_list.non_empty() {
      Some(actor_cell.dead_letter_mailbox())
    } else {
      None
    };
    while message_list.non_empty() {
      {
        let system_message = message_list.head().clone().unwrap();
        let mut system_message_guard = system_message.lock().unwrap();
        system_message_guard.unlink();
        let self_ref = actor_cell.actor_ref.clone().to_any();
        if let Some(dlm) = dead_letter_mailbox.as_mut() {
          dlm.system_enqueue(self_ref, &mut system_message_guard);
        }
      }
      message_list = message_list.tail();
    }
    if error_msg.is_some() {
      panic!("{}", error_msg.unwrap())
    }
  }
}

impl<Msg: Message> SystemMessageQueueReaderBehavior for SystemMailbox<Msg> {
  fn has_system_messages(&self) -> bool {
    let current_list = self.system_queue_get();
    let head_arc_opt = current_list.head();
    head_arc_opt
      .map(|head_arc| {
        let system_message_guard = head_arc.lock().unwrap();
        match &*system_message_guard {
          SystemMessageEntry {
            message: SystemMessage::NoMessage { .. },
            ..
          } => false,
          _ => true,
        }
      })
      .unwrap_or(false)
  }

  fn system_drain(&mut self, new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    loop {
      let current_list = self.system_queue_get();
      let head_arc_opt = current_list.head();
      if head_arc_opt.iter().any(|head_arc| {
        let system_message_guard = head_arc.lock().unwrap();
        system_message_guard.is_no_message()
      }) {
        return EarliestFirstSystemMessageList::new(None);
      } else if self.sender().system_queue_put(&current_list, new_contents) {
        return current_list.reverse();
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct SystemMailboxSender<Msg: Message> {
  underlying: SystemMailbox<Msg>,
  _phantom: std::marker::PhantomData<Msg>,
}

impl<Msg: Message> SystemMailboxSender<Msg> {
  fn system_queue_put(&mut self, old: &LatestFirstSystemMessageList, new: &LatestFirstSystemMessageList) -> bool {
    let same = match (old.head(), new.head()) {
      (Some(v1_arc), Some(v2_arc)) => {
        if Arc::ptr_eq(&v1_arc, &v2_arc) {
          true
        } else {
          let v1_guard = v1_arc.lock().unwrap();
          let v2_guard = v2_arc.lock().unwrap();
          &*v1_guard == &*v2_guard
        }
      }
      (None, None) => true,
      _ => false,
    };
    if same {
      return true;
    }

    let mut system_message_opt_guard = self.underlying.system_message_opt.lock().unwrap();
    let result = match new.head() {
      Some(arc) => {
        let system_message_guard = arc.lock().unwrap();
        *system_message_opt_guard = Some(system_message_guard.clone());
        true
      }
      None => {
        *system_message_opt_guard = None;
        true
      }
    };
    result
  }
}

impl<Msg: Message> SystemMessageQueueWriterBehavior<Msg> for SystemMailboxSender<Msg> {
  fn system_enqueue(&mut self, receiver: ActorRef<Msg>, message: &mut SystemMessageEntry) {
    let current_list = self.underlying.system_queue_get();
    let head_arc_opt = current_list.head();
    if head_arc_opt.iter().any(|head_arc| {
      let system_message_guard = head_arc.lock().unwrap();
      system_message_guard.is_no_message()
    }) {
      // TODO: デッドレターに送る
      ()
    } else {
      if !self.system_queue_put(&current_list.clone(), &current_list.prepend(message.clone())) {
        // putに失敗した場合、やり直すが、実際には発生しない
        message.unlink();
        self.system_enqueue(receiver, message);
      }
    }
  }
}
