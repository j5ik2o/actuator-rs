use num_enum::TryFromPrimitive;

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
