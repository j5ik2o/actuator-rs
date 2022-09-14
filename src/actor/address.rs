use mur3::{Hasher128, Hasher32};
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

impl Address {
  pub fn new(protocol: &str, system: &str) -> Self {
    Self {
      protocol: protocol.to_owned(),
      system: system.to_owned(),
      host: None,
      port: None,
    }
  }

  pub fn new_with_host_port(protocol: &str, system: &str, host: Option<&str>, port: Option<u32>) -> Self {
    match (host, port) {
      (None, Some(..)) => {
        panic!("Only ports cannot be specified.")
      }
      _ => {}
    }
    Self {
      protocol: protocol.to_owned(),
      system: system.to_owned(),
      host: host.map(|s| s.to_owned()),
      port,
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
}

#[cfg(test)]
mod tests {
  use crate::actor::address::Address;

  #[test]
  fn test_to_string_protocol_system() {
    let address = Address::new("tcp", "test");
    println!("to_string = {}", address.to_string());
    assert_eq!(address.to_string(), "tcp://test")
  }

  #[test]
  fn test_to_string_protocol_system_host_port() {
    let address = Address::new_with_host_port("tcp", "test", Some("host1"), Some(8080));
    println!("to_string = {}", address.to_string());
    assert_eq!(address.to_string(), "tcp://test@host1:8080")
  }

  #[test]
  fn test_hash_code() {
    let address = Address::new("tcp", "test");
    let hash_code = address.hash_code();
    println!("hash_code = {}", hash_code)
  }
}
