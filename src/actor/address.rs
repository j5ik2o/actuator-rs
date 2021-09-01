use std::fmt::{Formatter, Write};
use std::fmt;

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Address {
  pub protocol: String,
  pub system: String,
  pub host: Option<String>,
  pub port: Option<u16>,
}

impl fmt::Display for Address {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut result = format!("{}://{}", self.protocol, self.system);
    result = match &self.host {
      Some(h) => format!("{}@{}", result, h),
      None => result,
    };
    result = match self.port {
      Some(p) => format!("{}:{}", result, p),
      None => result,
    };
    write!(f, "{}", result)
  }
}

impl From<(String, String)> for Address {
  fn from((protocol, system): (String, String)) -> Self {
    Self {
      protocol,
      system,
      host: None,
      port: None,
    }
  }
}

impl From<(String, String, String, u16)> for Address {
  fn from((protocol, system, host, port): (String, String, String, u16)) -> Self {
    Self {
      protocol,
      system,
      host: Some(host),
      port: Some(port),
    }
  }
}

pub static INVALID_HOST_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());

impl Address {
  pub fn has_local_scope(&self) -> bool {
    self.host.is_none()
  }

  pub fn has_global_scope(&self) -> bool {
    self.host.is_some()
  }

  pub fn host_port(&self) -> String {
    let s = &self.to_string()[..self.protocol.len() + 1];
    String::from(s)
  }

  pub fn has_invalid_host_characters(&self) -> bool {
    match &self.host {
      Some(h) => INVALID_HOST_REGEX.is_match(h),
      None => false,
    }
  }

  pub fn check_host_characters(&self) {
    if self.has_invalid_host_characters() {
      panic!(
        "Using invalid host characters '{}' in the Address is not allowed.",
        self.host.as_ref().unwrap()
      )
    }
  }
}
