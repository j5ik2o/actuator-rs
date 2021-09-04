//
// pub struct Dispatcher<M: Message> {
//   msg: M,
// }
//
// impl<M: Message> Dispatcher<M> {
//   // fn register_for_execution(&self, _mbox: MailboxSender<M>) {}
//   //
//   // pub fn dispatch(&self, receiver: Arc<dyn ActorRef>, invocation: Envelope<M>) {
//   //   let mailbox_sender = receiver.mailbox_sender().clone();
//   //   mailbox_sender.try_enqueue(receiver, invocation).unwrap();
//   //   self.register_for_execution(mailbox_sender);
//   // }
// }
