use crate::kernel::dispatcher::Dispatcher;
use crate::kernel::queue::*;
use crate::kernel::{Envelope, Message};
#[allow(dead_code)]
pub enum MailboxStatus {
    Open,
    Closed,
    Scheduled,
    ShouldScheduleMask,
    ShouldNotProcessMask,
    SuspendMask,
    SuspendUnit,
}

#[allow(dead_code)]
pub struct Mailbox<Msg: Message> {
    limit: u32,
    queue: QueueReader<Msg>,
}

impl<Msg: Message> Mailbox<Msg> {
    pub fn new(limit: u32, queue: QueueReader<Msg>) -> Self {
        Self { limit, queue }
    }

    pub fn dequeue(&self) -> Envelope<Msg> {
        self.queue.dequeue()
    }

    pub fn try_dequeue(&self) -> Result<Envelope<Msg>, QueueEmpty> {
        self.queue.try_dequeue()
    }

    pub fn non_empty(&self) -> bool {
        self.queue.non_empty()
    }

    pub fn is_empty(&self) -> bool {
        !self.non_empty()
    }
}

pub fn mailbox<Msg: Message>(limit: u32) -> (Dispatcher<Msg>, Mailbox<Msg>) {
    let (qw, qr) = new_queue::<Msg>();
    let dispatcher = Dispatcher::new(qw);
    let mailbox = Mailbox::new(limit, qr);
    (dispatcher, mailbox)
}
