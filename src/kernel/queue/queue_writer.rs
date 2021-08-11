use crate::kernel::{Envelope, Message};
use std::sync::mpsc::Sender;

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
    pub fn new(tx: Sender<Envelope<Msg>>) -> Self {
        Self { tx }
    }

    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.tx
            .send(msg)
            .map(|_| ())
            .map_err(|e| EnqueueError { msg: e.0 })
    }
}
