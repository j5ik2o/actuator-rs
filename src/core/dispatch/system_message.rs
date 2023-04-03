use crate::core::actor::actor_ref::ActorRef;

use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::earliest_first_system_message_list::EarliestFirstSystemMessageList;
use crate::core::dispatch::system_message::latest_first_system_message_list::LatestFirstSystemMessageList;

use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;

use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub(crate) mod earliest_first_system_message_list;
pub(crate) mod latest_first_system_message_list;
pub(crate) mod system_message;
pub(crate) mod system_message_entry;
pub(crate) mod system_message_list;

pub trait SystemMessageQueueWriterBehavior<Msg: Message>: Debug {
  fn system_enqueue(&mut self, receiver: ActorRef<Msg>, message: &mut SystemMessageEntry);
}

pub trait SystemMessageQueueReaderBehavior: Debug {
  fn has_system_messages(&self) -> bool;
  fn system_drain(&mut self, new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList;
}

fn size_inner(head: Option<Arc<Mutex<SystemMessageEntry>>>, acc: usize) -> usize {
  if head.is_none() {
    acc
  } else {
    size_inner(
      head.and_then(|e| {
        let inner = e.lock().unwrap();
        inner.next.clone()
      }),
      acc + 1,
    )
  }
}

fn reverse_inner(
  head: Option<Arc<Mutex<SystemMessageEntry>>>,
  acc: Option<Arc<Mutex<SystemMessageEntry>>>,
) -> Option<Arc<Mutex<SystemMessageEntry>>> {
  match head {
    None => acc,
    Some(head_arc) => {
      let head_arc_cloned = head_arc.clone();
      let next = {
        let mut head_inner = head_arc.lock().unwrap();
        let next = head_inner.next.clone();
        head_inner.set_next(acc);
        next
      };
      reverse_inner(next, Some(head_arc_cloned))
    }
  }
}

pub const LNIL: Lazy<LatestFirstSystemMessageList> = Lazy::new(|| LatestFirstSystemMessageList::new(None));

pub const ENIL: Lazy<EarliestFirstSystemMessageList> = Lazy::new(|| EarliestFirstSystemMessageList::new(None));
