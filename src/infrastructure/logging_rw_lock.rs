use std::fmt::Debug;
use std::sync::RwLock;

pub struct LoggingRwLock<T: Debug> {
  pub(crate) inner: RwLock<T>,
  name: &'static str,
  log_output: bool,
  is_try: bool,
}

impl<T: Debug> LoggingRwLock<T> {
  pub fn new(name: &'static str, data: T) -> Self {
    LoggingRwLock {
      inner: RwLock::new(data),
      name,
      log_output: true,
      is_try: false,
    }
  }

  pub fn write_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::sync::LockResult<std::sync::RwLockWriteGuard<T>> {
    if self.log_output {
      log::debug!(
        "Attempting to write: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    if self.is_try {
      let guard = self.inner.try_write();
      match guard {
        Ok(guard) => {
          if self.log_output {
            log::debug!(
              "Write acquired: {} by {}:{} at {}:{}",
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
            "Write failed: {} by {}:{} at {}:{}",
            self.name, function_name, module_path, file, line,
          );
        }
      }
    } else {
      let guard = self.inner.write().unwrap();
      if self.log_output {
        log::debug!(
          "Write acquired: {} by {}:{} at {}:{}",
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

  pub fn read_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::sync::LockResult<std::sync::RwLockReadGuard<T>> {
    if self.log_output {
      log::debug!(
        "Attempting to read: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    if self.is_try {
      let guard = self.inner.try_read();
      match guard {
        Ok(guard) => {
          if self.log_output {
            log::debug!(
              "Read acquired: {} by {}:{} at {}:{}",
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
            "Read failed: {} by {}:{} at {}:{}",
            self.name, function_name, module_path, file, line,
          );
        }
      }
    } else {
      let guard = self.inner.read().unwrap();
      if self.log_output {
        log::debug!(
          "Read acquired: {} by {}:{} at {}:{}",
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
macro_rules! read_lock_with_log {
  ($ref_cell:expr, $fname:expr) => {
    $ref_cell
      .read_with_info($fname, module_path!(), file!(), line!())
      .unwrap()
  };
}

#[macro_export]
macro_rules! write_lock_with_log {
  ($ref_cell:expr, $fname:expr) => {
    $ref_cell
      .write_with_info($fname, module_path!(), file!(), line!())
      .unwrap()
  };
}

impl<T: Debug> Drop for LoggingRwLock<T> {
  fn drop(&mut self) {
    if self.log_output {
      log::debug!("dropping: name = {}", self.name);
    }
  }
}
