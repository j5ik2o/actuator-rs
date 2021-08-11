use crate::kernel::{Message, Envelope};
use std::sync::mpsc::{channel, Sender, Receiver};

pub fn queue<Msg: Message>() -> (QueueWriter<Msg>, QueueReader<Msg>) {
    let (tx, rx) = channel::<Envelope<Msg>>();
    let qw = QueueWriter { tx };
    let qr = QueueReader { rx };
    (qw, qr)
}

#[derive(Clone)]
pub struct QueueWriter<Msg: Message> {
    tx: Sender<Envelope<Msg>>,
}

#[derive(Clone, Debug)]
pub struct EnqueueError<T> {
    pub msg: T,
}

pub type EnqueueResult<Msg> = Result<(), EnqueueError<Envelope<Msg>>>;

impl<Msg: Message> QueueWriter<Msg> {
    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.tx
            .send(msg)
            .map(|_| ())
            .map_err(|e| EnqueueError { msg: e.0 })
    }
}

pub struct QueueReader<Msg: Message> {
    rx: Receiver<Envelope<Msg>>,
}

pub struct QueueEmpty;
pub type DequeueResult<Msg> = Result<Msg, QueueEmpty>;

impl<Msg: Message> QueueReader<Msg> {
    #[allow(dead_code)]
    pub fn dequeue(&self) -> Envelope<Msg> {
        self.rx.recv().unwrap()
    }

    pub fn try_dequeue(&self) -> DequeueResult<Envelope<Msg>> {
        self.rx.try_recv().map_err(|_| QueueEmpty)
    }

    pub fn has_messages(&self) -> bool {
        match self.rx.try_recv() {
            Ok(item) => true,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::queue::{queue, QueueEmpty};
    use crate::kernel::{Message, Envelope};
    use std::fmt::{Debug, Formatter};

    #[derive(Debug, Clone, PartialEq)]
    struct Counter(u32);

    impl Message for Counter {}

    #[test]
    fn test() {
        let (qw, qr) = queue::<Counter>();
        let expected_message = Envelope::new(Counter(1));
        qw.try_enqueue(expected_message.clone()).unwrap();

        let r = qr.try_dequeue().unwrap_or(Envelope::new(Counter(0)));
        assert_eq!(expected_message, r)

    }
}