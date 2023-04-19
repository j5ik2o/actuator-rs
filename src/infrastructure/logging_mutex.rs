use std::fmt::Debug;

use std::sync::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct LoggingMutex<T: Debug> {
  pub(crate) inner: Mutex<T>,
  name: &'static str,
  log_output: bool,
  is_try: bool,
}

impl<T: Debug> LoggingMutex<T> {
  pub fn new(name: &'static str, data: T) -> Self {
    LoggingMutex {
      inner: Mutex::new(data),
      name,
      log_output: false,
      is_try: false,
    }
  }

  pub fn lock_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::sync::LockResult<MutexGuard<T>> {
    if self.log_output {
      log::debug!(
        "Attempting to lock: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    if self.is_try {
      let guard = self.inner.try_lock();
      match guard {
        Ok(guard) => {
          if self.log_output {
            log::debug!(
              "Lock acquired: {} by {}:{} at {}:{}",
              self.name,
              function_name,
              module_path,
              file,
              line,
            );
          }
          return Ok(guard);
        }
        Err(_err) => {
          panic!(
            "Lock failed: {} by {}:{} at {}:{}",
            self.name, function_name, module_path, file, line,
          );
        }
      }
    } else {
      let guard = self.inner.lock().unwrap();
      if self.log_output {
        log::debug!(
          "Lock acquired: {} by {}:{} at {}:{}",
          self.name,
          function_name,
          module_path,
          file,
          line,
        );
      }
      return Ok(guard);
    }
  }
}

#[macro_export]
macro_rules! mutex_lock_with_log {
  ($mutex:expr, $fname:expr) => {
    $mutex.lock_with_info($fname, module_path!(), file!(), line!()).unwrap()
  };
}

impl<T: Debug> Drop for LoggingMutex<T> {
  fn drop(&mut self) {
    if self.log_output {
      log::debug!("Lock dropped: name = {}", self.name);
    }
  }
}
