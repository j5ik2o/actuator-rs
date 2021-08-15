use crate::kernel::{new_mailbox, Message};

#[derive(Debug, Clone, PartialEq)]
struct Counter(u32);

impl Message for Counter {}

#[test]
fn test_is_scheduled() {
  let mailbox1 = new_mailbox::<Counter>(2);
  assert!(!mailbox1.is_scheduled());
  mailbox1.set_as_scheduled();
  assert!(mailbox1.is_scheduled());
}
