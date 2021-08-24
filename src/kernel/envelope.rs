use crate::kernel::message::Message;
use crate::actor::actor_ref::InternalActorRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<M: Message> {
    pub message: M,
    pub sender: Option<InternalActorRef>
}

impl<M: Message> Envelope<M> {
    pub fn new(value: M, sender: Option<InternalActorRef>) -> Self {
        Self { message: value, sender }
    }
}