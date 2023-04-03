use std::cell::RefCell;
use std::fmt::Debug;

pub struct LoggingRefCell<T: Debug> {
  inner: RefCell<T>,
  name: &'static str,
  log_output: bool,
}

impl<T: Debug> LoggingRefCell<T> {
  pub fn new(name: &'static str, value: T) -> Self {
    Self {
      inner: RefCell::new(value),
      name,
      log_output: true,
    }
  }

  pub fn borrow_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::cell::Ref<T> {
    if self.log_output {
      log::debug!(
        "Attempting to borrow: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    match self.inner.try_borrow() {
      Ok(guard) => {
        if self.log_output {
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
  }

  pub fn borrow_mut_with_info(
    &self,
    function_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
  ) -> std::cell::RefMut<T> {
    if self.log_output {
      log::debug!(
        "Attempting to borrow_mut: {} by {}:{} at {}:{}",
        self.name,
        function_name,
        module_path,
        file,
        line,
      );
    }
    match self.inner.try_borrow_mut() {
      Ok(guard) => {
        if self.log_output {
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
    if self.log_output {
      log::debug!("dropping: name = {}", self.name);
    }
  }
}
