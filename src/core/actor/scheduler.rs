use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex,
};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::{runtime, task};

use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::dispatch::message::Message;
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum CancellableError {
  #[error("Task handle not set")]
  HandleNotSet,
  #[error(transparent)]
  JoinError(#[from] task::JoinError),
}

#[derive(Debug)]
struct CancellableInner {
  task_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct Cancellable {
  inner: Arc<Mutex<CancellableInner>>,
  cancelled: Arc<AtomicBool>,
}

impl Cancellable {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(CancellableInner { task_handle: None })),
      cancelled: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn cancel(&self) {
    self.cancelled.store(true, Ordering::SeqCst);
  }

  pub fn is_cancelled(&self) -> bool {
    self.cancelled.load(Ordering::SeqCst)
  }

  pub fn set_task_handle(&mut self, handle: JoinHandle<()>) {
    let mut inner = self.inner.lock().unwrap();
    inner.task_handle = Some(handle);
  }

  pub fn join(self) -> Result<(), CancellableError> {
    let runtime = runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let mut inner = self.inner.lock().unwrap();
    let h = inner.task_handle.take();
    match h {
      Some(handle) => runtime.block_on(handle).map_err(CancellableError::from),
      None => Err(CancellableError::HandleNotSet),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Scheduler {
  tick_duration: Duration,
}

impl Scheduler {
  pub fn new(tick_duration: Duration) -> Self {
    Self { tick_duration }
  }

  pub fn schedule_with_fixed_delay_to_actor_ref<U: Message>(
    &self,
    runtime: Arc<Runtime>,
    initial_delay: Duration,
    delay: Duration,
    receiver: ActorRef<U>,
    message: U,
  ) -> Cancellable {
    self.schedule_with_fixed_delay(runtime, initial_delay, delay, move || {
      receiver.clone().tell(message.clone());
    })
  }

  pub fn schedule_with_fixed_delay<F>(
    &self,
    runtime: Arc<Runtime>,
    initial_delay: Duration,
    delay: Duration,
    f: F,
  ) -> Cancellable
  where
    F: Fn() + Send + 'static, {
    let cancellable = Cancellable::new();
    let cancellable_clone = cancellable.clone();
    let tick_duration = self.tick_duration.clone();
    let join_handle = runtime.spawn(async move {
      log::debug!("Task started!");
      let mut interval = tokio::time::interval(tick_duration);
      let total_intervals = initial_delay.as_millis() / tick_duration.as_millis();
      log::debug!("interval: {}", total_intervals);
      log::debug!("total_intervals: {}", total_intervals);

      for _ in 0..total_intervals {
        interval.tick().await;

        if cancellable_clone.is_cancelled() {
          log::debug!("Task cancelled!");
          return;
        }
      }

      f();

      let mut interval = tokio::time::interval(tick_duration);
      let total_intervals = delay.as_millis() / tick_duration.as_millis();
      log::debug!("interval: {}", total_intervals);
      log::debug!("total_intervals: {}", total_intervals);

      loop {
        for _ in 0..total_intervals {
          interval.tick().await;

          if cancellable_clone.is_cancelled() {
            log::debug!("Task cancelled!");
            return;
          }
        }

        f();
      }
    });

    let mut cancellable_with_handle = cancellable;
    cancellable_with_handle.set_task_handle(join_handle);
    cancellable_with_handle
  }

  pub fn schedule_once_to_actor_ref<U: Message>(
    &self,
    runtime: Arc<Runtime>,
    delay: Duration,
    receiver: ActorRef<U>,
    message: U,
  ) -> Cancellable {
    self.schedule_once(runtime, delay, move || {
      log::debug!("Sending message to actor: {:?}", receiver.clone());
      receiver.clone().tell(message.clone());
    })
  }

  pub fn schedule_once<F>(&self, runtime: Arc<Runtime>, delay: Duration, f: F) -> Cancellable
  where
    F: Fn() + Send + 'static, {
    let cancellable = Cancellable::new();
    let cancellable_clone = cancellable.clone();
    let tick_duration = self.tick_duration.clone();
    let join_handle = runtime.spawn(async move {
      log::debug!("Task started!");
      let mut interval = tokio::time::interval(tick_duration);
      let total_intervals = delay.as_millis() / tick_duration.as_millis();
      log::debug!("interval: {}", total_intervals);
      log::debug!("total_intervals: {}", total_intervals);

      for _ in 0..total_intervals {
        interval.tick().await;
        if cancellable_clone.is_cancelled() {
          log::debug!("Task cancelled!");
          return;
        }
      }

      f();
      log::debug!("Task finished!");
    });

    let mut cancellable_with_handle = cancellable;
    cancellable_with_handle.set_task_handle(join_handle);
    cancellable_with_handle
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread;

  #[test]
  fn test_schedule_with_fixed_delay() {
    let delay = Duration::from_millis(1000);
    let _start = tokio::time::Instant::now();
    let runtime = Arc::new(Runtime::new().unwrap());

    let cancellable = Scheduler::new(Duration::from_millis(10)).schedule_with_fixed_delay(
      runtime.clone(),
      delay.clone(),
      delay,
      move || {
        log::debug!(">>> Task executed!");
      },
    );

    thread::sleep(Duration::from_secs(3));

    cancellable.cancel();

    // タスクが完了するまで待ちます。
    let result = cancellable.join();
    assert!(result.is_ok());

    // let actual = start.elapsed();
    //
    // let lower_bound = delay - Duration::from_millis(100);
    // let upper_bound = delay + Duration::from_millis(100);
    //
    // assert!(
    //     actual >= lower_bound && actual <= upper_bound,
    //     "actual = {:?}, expected between {:?} and {:?}",
    //     actual, lower_bound, upper_bound
    // );
  }

  #[test]
  fn test_schedule_once() {
    let delay = Duration::from_millis(1000);
    let start = tokio::time::Instant::now();
    let runtime = Arc::new(Runtime::new().unwrap());

    let cancellable = Scheduler::new(Duration::from_millis(10)).schedule_once(runtime.clone(), delay, || {
      log::debug!(">>> Task executed!");
    });

    // タスクが完了するまで待ちます。
    let result = cancellable.join();
    assert!(result.is_ok());

    let actual = start.elapsed();

    let lower_bound = delay - Duration::from_millis(100);
    let upper_bound = delay + Duration::from_millis(100);

    assert!(
      actual >= lower_bound && actual <= upper_bound,
      "actual = {:?}, expected between {:?} and {:?}",
      actual,
      lower_bound,
      upper_bound
    );
  }

  // #[test]
  // fn test_schedule_once_no_handle() {
  //   let mut cancellable = Cancellable::new();
  //
  //   // タスクが実行されないように task_handle を None に設定します。
  //   cancellable.task_handle = None;
  //
  //   let result = cancellable.join();
  //
  //   // CancellableError::HandleNotSet エラーが返されることを確認します。
  //   assert!(matches!(result, Err(CancellableError::HandleNotSet)));
  // }
}
