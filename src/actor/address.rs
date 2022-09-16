use mur3::{Hasher128, Hasher32};
use once_cell::sync::Lazy;
use oni_comb_uri_rs::models::uri::Uri;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::ptr::hash;

#[derive(Hash)]
pub struct Address {
  protocol: String,
  system: String,
  host: Option<String>,
  port: Option<u32>,
}

impl PartialEq<Self> for Address {
  fn eq(&self, other: &Self) -> bool {
    self.protocol == other.protocol && self.system == other.system && self.host == other.host && self.port == other.port
  }
}

impl PartialOrd for Address {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self == other {
      Some(Ordering::Equal)
    } else if self.protocol != other.protocol {
      self.system.partial_cmp(&other.protocol)
    } else if self.system != other.system {
      self.system.partial_cmp(&other.system)
    } else if self.host != other.host {
      self
        .host
        .as_ref()
        .unwrap_or(&"".to_owned())
        .partial_cmp(other.host.as_ref().unwrap_or(&"".to_owned()))
    } else if self.port != other.port {
      self.port.unwrap_or(0).partial_cmp(&other.port.unwrap_or(0))
    } else {
      None
    }
  }
}

impl fmt::Display for Address {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}://{}{}{}",
      self.protocol,
      self.system,
      self.host.as_ref().map(|s| format!("@{}", s)).unwrap_or("".to_owned()),
      self.port.map(|n| format!(":{}", n)).unwrap_or("".to_owned())
    )
  }
}

use regex::Regex;
static INVALID_HOST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"_[^.]*$").unwrap());

impl Address {
  pub fn new(protocol: &str, system: &str) -> Self {
    Self {
      protocol: protocol.to_owned(),
      system: system.to_owned(),
      host: None,
      port: None,
    }
  }

  pub fn new_with_host_port(protocol: &str, system: &str, host: &str, port: u32) -> Self {
    Self {
      protocol: protocol.to_owned(),
      system: system.to_owned(),
      host: Some(host.to_owned()),
      port: Some(port),
    }
  }

  pub fn has_local_scope(&self) -> bool {
    self.host.is_none()
  }

  pub fn has_global_scope(&self) -> bool {
    self.host.is_some()
  }

  pub fn hash_code(&self) -> u128 {
    let mut hasher = Hasher128::with_seed(0xcafebabe);
    self.hash(&mut hasher);
    let (u, d) = hasher.finish128();
    (u as u128) << 64 | d as u128
  }

  pub fn host_port(&self) -> String {
    self.to_string()[..self.protocol.len() + 3].to_owned()
  }

  pub fn has_invalid_host_characters(&self) -> bool {
    self
      .host
      .as_ref()
      .map(|s| INVALID_HOST_REGEX.is_match(&s))
      .unwrap_or(false)
  }

  fn check_host_characters(&self) {
    if self.has_invalid_host_characters() {
      panic!(
        "Using invalid host characters '{}' in the Address is not allowed.",
        self.host.as_ref().unwrap_or(&"".to_owned())
      )
    }
  }
}

fn rec(s: String, fragment: Option<String>, pos: usize, acc: Vec<String>) -> Vec<String> {
  if pos == 0 {
    acc
  } else {
    log::debug!("pos = {}", pos);
    let from = s[..pos - 1].rfind(|c| c == '/').map(|n| n as i32).unwrap_or(-1);
    let sub = &s[((from + 1) as usize)..pos];
    let l = if fragment.is_some() && acc.is_empty() {
      let mut t = vec![format!("{}#{}", sub, fragment.as_ref().unwrap())];
      let mut s = acc;
      t.append(&mut s);
      t
    } else {
      let mut t = vec![sub.to_owned()];
      let mut s = acc;
      t.append(&mut s);
      t
    };
    if from == -1 {
      l
    } else {
      rec(s, fragment, from as usize, l)
    }
  }
}

fn path_split(s: String, fragment: Option<String>) -> Vec<String> {
  rec(s.clone(), fragment, s.len(), Vec::new())
}

fn unapply_address(addr: String) -> Option<Vec<String>> {
  let uri = Uri::parse(&addr).unwrap();
  log::debug!("uri = {}", uri);
  if uri.is_absolute() {
    None
  } else {
    Some(path_split(
      uri.path().unwrap().to_string(),
      uri.fragment().map(|s| s.clone()),
    ))
  }
}

fn address_from_uri_string_unapply(uri_opt: Option<Uri>) {
  match uri_opt {
    None => None,
    Some(uri) if uri.schema(). => {}

  }

}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::actor::address::Address;
  use oni_comb_uri_rs::models::uri::Uri;
  use std::{env, panic};

  #[ctor::ctor]
  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    // env::set_var("RUST_LOG", "trace");
    let _ = env_logger::try_init();
  }

  #[test]
  fn test_path_split() {
    let r = path_split("aaa/bbb/ccc".to_owned(), Some("f".to_owned()));
    assert_eq!(r, vec!["aaa", "bbb", "ccc#f"]);
  }

  #[test]
  fn test_unapply_address() {
    let r = unapply_address("http://localhost/aaa/bbb/ccc#f".to_string()).unwrap();
    assert_eq!(r, vec!["aaa", "bbb", "ccc#f"]);
  }

  #[test]
  fn test_to_string_protocol_system() {
    let address = Address::new("tcp", "test");
    println!("to_string = {}", address.to_string());
    assert_eq!(address.to_string(), "tcp://test")
  }

  #[test]
  fn test_to_string_protocol_system_host_port() {
    let address = Address::new_with_host_port("tcp", "test", "host1", 8080);
    log::debug!("to_string = {}", address.to_string());
    assert_eq!(address.to_string(), "tcp://test@host1:8080")
  }

  #[test]
  fn test_hash_code() {
    let address = Address::new("tcp", "test");
    let hash_code = address.hash_code();
    println!("hash_code = {}", hash_code)
  }

  #[test]
  fn test_has_invalid_host_characters() {
    let addresses = [
      Address::new_with_host_port("actuator", "sys", "valid", 0),
      Address::new_with_host_port("actuator", "sys", "is_valid.org", 0),
      Address::new_with_host_port("actuator", "sys", "fu.is_valid.org", 0),
    ];
    assert!(!addresses.iter().all(|e| e.has_invalid_host_characters()));

    let addresses = [
      Address::new_with_host_port("actuator", "sys", "in_valid", 0),
      Address::new_with_host_port("actuator", "sys", "invalid._org", 0),
    ];
    assert!(addresses.iter().all(|e| e.has_invalid_host_characters()));
  }

  #[test]
  fn test_check_host_characters() {
    let addresses = [
      Address::new_with_host_port("actuator", "sys", "valid", 0),
      Address::new_with_host_port("actuator", "sys", "is_valid.org", 0),
      Address::new_with_host_port("actuator", "sys", "fu.is_valid.org", 0),
    ];
    addresses.iter().for_each(|e| e.check_host_characters());

    let addresses = [
      Address::new_with_host_port("actuator", "sys", "in_valid", 0),
      Address::new_with_host_port("actuator", "sys", "invalid._org", 0),
    ];
    let result = panic::catch_unwind(|| {
      addresses.iter().for_each(|e| e.check_host_characters());
    });
    assert!(result.is_err());

    let result = Uri::parse("http://localhost").unwrap();
    println!("{:?}", result);
    // let host_name = result.authority().unwrap().host_name();
    // println!("{}", host_name);
  }
}
