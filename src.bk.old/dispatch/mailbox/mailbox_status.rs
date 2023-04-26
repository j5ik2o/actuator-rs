use num_enum::TryFromPrimitive;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum MailboxStatus {
  Open = 0,
  Closed = 1,
  Scheduled = 2,
  ShouldScheduleMask = 3,
  ShouldNotProcessMask = !2,
  SuspendMask = !3,
  SuspendUnit = 4,
}

impl Display for MailboxStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "MailboxStatus::{}",
      match self {
        MailboxStatus::Open => "Open",
        MailboxStatus::Closed => "Closed",
        MailboxStatus::Scheduled => "Scheduled",
        MailboxStatus::ShouldScheduleMask => "ShouldScheduleMask",
        MailboxStatus::ShouldNotProcessMask => "ShouldNotProcessMask",
        MailboxStatus::SuspendMask => "SuspendMask",
        MailboxStatus::SuspendUnit => "SuspendUnit",
      }
    )
  }
}
