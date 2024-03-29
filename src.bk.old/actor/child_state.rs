use std::time::{Duration, Instant};

use crate::actor::actor_path::ActorPathBehavior;

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::ActorRefBehavior;
use crate::dispatch::message::Message;

#[derive(Debug, Clone)]
pub enum ChildState<Msg: Message> {
  ChildNameReserved,
  ChildRestartStats(ChildRestartStats<Msg>),
}

impl<Msg: Message> PartialEq for ChildState<Msg> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::ChildNameReserved, Self::ChildNameReserved) => true,
      (Self::ChildRestartStats(l), Self::ChildRestartStats(r)) => l == r,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChildRestartStats<Msg: Message> {
  pub(crate) child: ActorRefRef<Msg>,
  max_nr_of_retries_count: i32,
  restart_time_window_start_nanos: i64,
}

impl<Msg: Message> ChildRestartStats<Msg> {
  pub fn new(child: ActorRefRef<Msg>) -> Self {
    Self::new_with(child, 0, 0)
  }

  pub fn new_with(child: ActorRefRef<Msg>, max_nr_of_retries_count: i32, restart_time_window_start_nanos: i64) -> Self {
    Self {
      child,
      max_nr_of_retries_count,
      restart_time_window_start_nanos,
    }
  }

  pub fn uid(&self) -> u32 {
    self.child.path().uid()
  }

  pub fn request_restart_permission(&mut self, retries_window: (Option<i32>, Option<i32>)) -> bool {
    match retries_window {
      (Some(retires), _) if retires < 1 => false,
      (Some(retires), None) => {
        self.max_nr_of_retries_count += 1;
        self.max_nr_of_retries_count <= retires
      }
      (x, Some(window)) => self.retries_in_window_okay(if x.is_some() { x.unwrap() } else { 1 }, window),
      (None, _) => true,
    }
  }

  fn retries_in_window_okay(&mut self, retries: i32, window: i32) -> bool {
    let retries_done = self.max_nr_of_retries_count + 1;
    let now = Instant::now().elapsed().as_nanos() as i64;
    let window_start = if self.restart_time_window_start_nanos == 0 {
      self.restart_time_window_start_nanos = now;
      now
    } else {
      self.restart_time_window_start_nanos
    };
    let inside_window = (now - window_start) as u128 <= Duration::from_millis(window as u64).as_nanos();
    if inside_window {
      self.max_nr_of_retries_count = retries_done;
      retries_done <= retries
    } else {
      self.max_nr_of_retries_count = 1;
      self.restart_time_window_start_nanos = now;
      true
    }
  }
}
