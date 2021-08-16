use crate::kernel::{Message, Envelope, QueueWriter};
use std::sync::{Mutex, Arc};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct QueueWriterInVecQueue<M: Message> {
    inner: Arc<Mutex<QueueWriterInVecQueueInner<M>>>,
}

struct QueueWriterInVecQueueInner<M: Message> {
    queue: VecDeque<Envelope<M>>,
}

impl<M: Message> QueueWriterInVecQueue<M> {
    pub fn new(inner: Arc<VecDeque<Envelope<M>>>) -> Self {
        let q = Arc::try_unwrap(inner).unwrap();
        let inner = Arc::from(Mutex::new(QueueWriterInVecQueueInner { queue: q }));
        Self { inner }
    }
}

impl<M: Message> QueueWriter<M> for QueueWriterInVecQueue<M> {
    fn try_enqueue(&self, msg: Envelope<M>) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push_back(msg);
        Ok(())
    }
}
