#[allow(dead_code)]
mod actor_cell;
mod dispatcher;
mod mailbox;
mod queue;

use std::fmt::Debug;
use crate::kernel::dispatcher::Dispatcher;
use crate::kernel::mailbox::Mailbox;
use crate::kernel::queue::new_queue;

pub trait Message: Debug + Clone + Send + 'static + PartialEq {}
// impl<T: Debug + Clone + Send + 'static> Message for T {}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<M: Message> {
    //  pub sender: Option<BasicActorRef>,
    pub msg: M,
}

impl<M: Message> Envelope<M> {
    pub fn new(value: M) -> Self {
        Self { msg: value }
    }
}


pub fn new_mailbox<M: Message>(limit: u32) -> (Dispatcher<M>, Mailbox<M>) {
    let (qw, qr) = new_queue();
    let dispatcher = Dispatcher::new(qw);
    let mailbox = Mailbox::new(limit, qr);
    (dispatcher, mailbox)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Counter(u32);

    impl Message for Counter {}

    #[test]
    fn test_new_mailbox() {
        let (qw, qr) = new_mailbox::<Counter>(1);
        let expected_message = Envelope::new(Counter(1));
        qw.try_enqueue(expected_message.clone()).unwrap();

        let r = qr.try_dequeue().unwrap_or(Envelope::new(Counter(0)));
        assert_eq!(expected_message, r)
    }
}