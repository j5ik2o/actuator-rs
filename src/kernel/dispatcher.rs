use crate::kernel::queue::{EnqueueResult, QueueWriter};
use crate::kernel::{Envelope, Message};

#[derive(Clone)]
pub struct Dispatcher<Msg: Message> {
    queue: QueueWriter<Msg>,
}

impl<Msg: Message> Dispatcher<Msg> {
    pub fn new(queue: QueueWriter<Msg>) -> Self {
        Self { queue }
    }

    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.queue.try_enqueue(msg)
    }
}

unsafe impl<Msg: Message> Send for Dispatcher<Msg> {}

unsafe impl<Msg: Message> Sync for Dispatcher<Msg> {}
