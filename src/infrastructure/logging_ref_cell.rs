use std::cell::RefCell;
use std::fmt::Debug;

pub struct LoggingRefCell<T: Debug> {
  inner: RefCell<T>,
  name: String,
  borrow_log_output: bool,
  drop_log_output: bool,
  is_try: bool,
}

impl<T: Debug> LoggingRefCell<T> {
  pub fn new(name: &str, value: T) -> Self {
    Self {
      inner: RefCell::new(value),
      name: name.to_string(),
      borrow_log_output: false,
      drop_log_output: true,
      is_try: false,
    }
  }

  pub fn borrow_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::cell::Ref<T> {
    if self.borrow_log_output {
      log::debug!(
        "Attempting to borrow: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    if self.is_try {
      match self.inner.try_borrow() {
        Ok(guard) => {
          if self.borrow_log_output {
            log::debug!(
              "Borrow acquired: {} by {}:{} at {}:{}",
              self.name,
              function_name,
              module_path,
              file,
              line,
            );
          }
          return guard;
        }
        Err(err) => {
          panic!(
            "Borrow failed: {} by {}:{} at {}:{}, err: {:?}",
            self.name, function_name, module_path, file, line, err
          );
        }
      }
    } else {
      let guard = self.inner.borrow();
      if self.borrow_log_output {
        log::debug!(
          "Borrow acquired: {} by {}:{} at {}:{}",
          self.name,
          function_name,
          module_path,
          file,
          line,
        );
      }
      return guard;
    }
  }

  pub fn borrow_mut_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::cell::RefMut<T> {
    if self.borrow_log_output {
      log::debug!(
        "Attempting to borrow_mut: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    if self.is_try {
      match self.inner.try_borrow_mut() {
        Ok(guard) => {
          if self.borrow_log_output {
            log::debug!(
              "Borrow_mut acquired: {} by {}:{} at {}:{}",
              self.name,
              function_name,
              module_path,
              file,
              line,
            );
          }
          return guard;
        }
        Err(err) => {
          panic!(
            "Borrow_mut failed: {} by {}:{} at {}:{}, err: {:?}",
            self.name, function_name, module_path, file, line, err
          );
        }
      }
    } else {
      let guard = self.inner.borrow_mut();
      if self.borrow_log_output {
        log::debug!(
          "Borrow_mut acquired: {} by {}:{} at {}:{}",
          self.name,
          function_name,
          module_path,
          file,
          line,
        );
      }
      return guard;
    }
  }
}

#[macro_export]
macro_rules! borrow_with_log {
  ($ref_cell:expr, $fname:expr) => {
    $ref_cell.borrow_with_info($fname, module_path!(), file!(), line!())
  };
}

#[macro_export]
macro_rules! borrow_mut_with_log {
  ($ref_cell:expr, $fname:expr) => {
    $ref_cell.borrow_mut_with_info($fname, module_path!(), file!(), line!())
  };
}

impl<T: Debug> Drop for LoggingRefCell<T> {
  fn drop(&mut self) {
    if self.drop_log_output {
      log::debug!(":::::>>> Dropped RefCell: name = {}", self.name);
    }
  }
}
