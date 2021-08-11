mod queue;
mod mailbox;
mod actor_cell;

use std::fmt::Debug;
use std::sync::mpsc::{Sender, Receiver, channel};

pub trait Message: Debug + Clone + Send + 'static + PartialEq {}
// impl<T: Debug + Clone + Send + 'static> Message for T {}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<T: Message> {
    //  pub sender: Option<BasicActorRef>,
    pub msg: T,
}

impl<T: Message> Envelope<T> {
    pub fn new(value: T) -> Self {
        Self { msg: value }
    }
}
