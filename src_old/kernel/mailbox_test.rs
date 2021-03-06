use crate::kernel::{new_mailbox, MailboxType};

use std::{env, panic};

#[derive(Debug, Clone, PartialEq)]
struct Counter(u32);

// impl Message for Counter {}

#[test]
fn test_is_scheduled() {
  fn test(mailbox_type: MailboxType) {
    let mut mailbox1 = new_mailbox::<Counter>(mailbox_type, 2);
    mailbox1.set_as_idle();
    assert!(!mailbox1.is_scheduled());
    mailbox1.set_as_scheduled();
    assert!(mailbox1.is_scheduled());
    mailbox1.set_as_idle();
    assert!(!mailbox1.is_scheduled());
  }

  run_test(|| {
    test(MailboxType::MPSC);
    test(MailboxType::VecQueue);
  });
}

#[test]
fn test_is_closed() {
  fn test(mailbox_type: MailboxType) {
    let mut mailbox1 = new_mailbox::<Counter>(mailbox_type, 2);
    mailbox1.set_as_idle();
    assert!(!mailbox1.is_closed());
    mailbox1.become_closed();
    assert!(mailbox1.is_closed());
    mailbox1.set_as_idle();
    assert!(mailbox1.is_closed());
  }
  run_test(|| {
    test(MailboxType::MPSC);
    test(MailboxType::VecQueue);
  });
}

#[test]
fn test_is_suspend() {
  fn test(mailbox_type: MailboxType) {
    let mut mailbox1 = new_mailbox::<Counter>(mailbox_type, 2);
    mailbox1.set_as_idle();
    assert!(!mailbox1.is_suspend());
    assert_eq!(mailbox1.suspend_count(), 0);
    mailbox1.suspend();
    assert_eq!(mailbox1.suspend_count(), 1);
    mailbox1.suspend();
    assert_eq!(mailbox1.suspend_count(), 2);
    mailbox1.suspend();
    assert_eq!(mailbox1.suspend_count(), 3);
    assert!(mailbox1.is_suspend());
    mailbox1.resume();
    assert_eq!(mailbox1.suspend_count(), 2);
    mailbox1.resume();
    assert_eq!(mailbox1.suspend_count(), 1);
    mailbox1.resume();
    assert_eq!(mailbox1.suspend_count(), 0);
    assert!(!mailbox1.is_suspend());
    mailbox1.set_as_idle();
    assert!(!mailbox1.is_suspend());
  }
  run_test(|| {
    test(MailboxType::MPSC);
    test(MailboxType::VecQueue);
  });
}

fn setup() {
  env::set_var("RUST_LOG", "debug");
  // env::set_var("RUST_LOG", "trace");
  logger::try_init();
}

fn teardown() {}

fn run_test<T>(test: T) -> ()
where
  T: FnOnce() -> () + panic::UnwindSafe,
{
  setup();

  let result = panic::catch_unwind(|| test());

  teardown();

  assert!(result.is_ok())
}
