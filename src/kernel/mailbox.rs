use crate::kernel::{Message, Envelope};
use crate::kernel::queue::{QueueReader, QueueEmpty, QueueWriter, EnqueueResult, queue};

pub enum MailboxStatus {
    Open,
    Closed,
    Scheduled,
    ShouldScheduleMask,
    ShouldNotProcessMask,
    SuspendMask,
    SuspendUnit,
}

pub struct Mailbox<Msg: Message> {
    limit: u32,
    queue: QueueReader<Msg>,
}

impl<Msg: Message> Mailbox<Msg> {
    pub fn dequeue(&self) -> Envelope<Msg> {
        self.queue.dequeue()
    }

    pub fn try_dequeue(&self) -> Result<Envelope<Msg>, QueueEmpty> {
        self.queue.try_dequeue()
    }

    pub fn has_messages(&self) -> bool {
        self.queue.has_messages()
    }
}

#[derive(Clone)]
pub struct Dispatcher<Msg: Message> {
    queue: QueueWriter<Msg>,
}

impl<Msg: Message> Dispatcher<Msg> {

    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.queue.try_enqueue(msg)
    }

}

unsafe impl<Msg: Message> Send for Dispatcher<Msg> {}

unsafe impl<Msg: Message> Sync for Dispatcher<Msg> {}

pub fn mailbox<Msg: Message>(limit: u32) -> (Dispatcher<Msg>, Mailbox<Msg>) {
    let (qw, qr) = queue::<Msg>();
    let dispatcher = Dispatcher { queue: qw };
    let mailbox = Mailbox { limit, queue: qr };
    (dispatcher, mailbox)
}