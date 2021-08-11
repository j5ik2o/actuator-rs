#[allow(dead_code)]
mod actor_cell;
mod dispatcher;
mod mailbox;
mod queue;

use std::fmt::Debug;

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
