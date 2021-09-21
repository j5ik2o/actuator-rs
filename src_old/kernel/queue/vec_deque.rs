use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::actor::actor_ref::ActorRef;
use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;
use crate::kernel::queue::{MessageSize, QueueReader, QueueWriter};

#[derive(Debug, Clone)]
pub struct QueueInVecQueue<M: Message> {
  inner: Arc<RwLock<QueueInVecQueueInner<M>>>,
}

#[derive(Debug)]
struct QueueInVecQueueInner<M: Message> {
  queue: VecDeque<Envelope<M>>,
}

impl<M: Message> QueueInVecQueue<M> {
  pub fn new(queue: VecDeque<Envelope<M>>) -> Self {
    let inner = Arc::from(RwLock::new(QueueInVecQueueInner { queue }));
    Self { inner }
  }
}

impl<M: Message> QueueReader<M> for QueueInVecQueue<M> {
  fn dequeue(&self) -> Envelope<M> {
    let mut inner = self.inner.write().unwrap();
    inner.queue.pop_front().unwrap()
  }

  fn try_dequeue(&self) -> anyhow::Result<Option<Envelope<M>>> {
    let mut inner = self.inner.write().unwrap();
    let result = inner.queue.pop_front();
    Ok(result)
  }

  fn non_empty(&self) -> bool {
    let inner = self.inner.read().unwrap();
    !inner.queue.is_empty()
  }

  fn number_of_messages(&self) -> MessageSize {
    let inner = self.inner.read().unwrap();
    MessageSize::Limit(inner.queue.len())
  }
}

impl<M: Message> QueueWriter<M> for QueueInVecQueue<M> {
  fn try_enqueue(
    &self,
    _receiver: Option<Arc<dyn ActorRef>>,
    msg: Envelope<M>,
  ) -> anyhow::Result<()> {
    let mut inner = self.inner.write().unwrap();
    inner.queue.push_back(msg);
    Ok(())
  }
}
