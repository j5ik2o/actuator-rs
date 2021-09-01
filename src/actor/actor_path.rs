use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;

use uri_rs::Uri;

use crate::actor::actor_cell::ActorCell;
use crate::actor::address::Address;
use fasthash::*;
use fasthash::murmur::Hash32;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone)]
pub enum ActorPath {
  Root {
    address: Address,
    name: String,
  },
  Child {
    parent: Arc<ActorPath>,
    name: String,
    uid: u32,
  },
}

impl Hash for ActorPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      ActorPath::Root { address, name } => {
        name.hash(state);
        address.hash(state);
      }
      ActorPath::Child { parent, name, uid } => {
        parent.hash(state);
        name.hash(state);
        uid.hash(state);
      }
    }
  }
}

impl PartialEq for ActorPath {
  fn eq(&self, other: &Self) -> bool {
    match self {
      ActorPath::Root { .. } => root_partial_cmp(self, other) == Some(Ordering::Equal),
      ActorPath::Child { .. } => child_partial_cmp(self, other) == Some(Ordering::Equal),
    }
  }
}

fn root_partial_cmp(me: &ActorPath, other: &ActorPath) -> Option<Ordering> {
  match other {
    ActorPath::Root { .. } => me.to_string().partial_cmp(&other.to_string()),
    ActorPath::Child { .. } => Some(Ordering::Greater),
  }
}

fn child_partial_cmp(me: &ActorPath, other: &ActorPath) -> Option<Ordering> {
  let mut cur_me = me;
  let mut cur_other = other;
  loop {
    if cur_me == cur_other {
      return Some(Ordering::Equal);
    } else if cur_me.is_root() {
      return root_partial_cmp(cur_me, cur_other);
    } else if other.is_root() {
      return match root_partial_cmp(cur_other, cur_me) {
        Some(Ordering::Greater) => Some(Ordering::Less),
        Some(Ordering::Less) => Some(Ordering::Greater),
        x => x,
      };
    } else {
      let x = cur_me
        .name_with_uid()
        .partial_cmp(&cur_other.name_with_uid());
      if x != Some(Ordering::Equal) {
        return x;
      }
      cur_me = cur_me.parent();
      cur_other = cur_other.parent();
    }
  }
}

impl PartialOrd for ActorPath {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match self {
      ActorPath::Root { .. } => root_partial_cmp(self, other),
      ActorPath::Child { .. } => child_partial_cmp(self, other),
    }
  }
}

impl Display for ActorPath {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let s = match self {
      ActorPath::Child { parent, name, uid } => {
        let uid_str = if *uid > 0 {
          format!("#{}", uid.to_string())
        } else {
          "".to_string()
        };
        if parent.is_child() {
          format!("{}/{}{}", parent.to_string(), name, uid_str)
        } else {
          format!("{}{}{}", parent.to_string(), name, uid_str)
        }
      }
      ActorPath::Root { address, name } => format!("{}{}", address.to_string(), name),
    };
    write!(f, "{}", s)
  }
}

impl ActorPath {

  fn address_from_uri_string(s: &str) -> Option<(Address, Vec<String>, Option<String>)> {
    Uri::parse(s)
      .ok()
      .and_then(|uri| Self::address_from_uri(uri))
  }

  fn address_from_uri(uri: Uri) -> Option<(Address, Vec<String>, Option<String>)> {
    let scheme = uri.schema().to_string();
    let user_name = uri
      .authority()
      .and_then(|authority| authority.user_info().map(|ui| ui.user_name()));
    let host_name = uri
      .authority()
      .map(|authority| authority.host_name().to_string());
    let port = uri.authority().and_then(|authority| authority.port());
    match (scheme, user_name, host_name, port) {
      (s, Some(un), Some(h), Some(p)) => Some((
        Address::from((s.to_string(), un.to_string(), h.to_string(), p)),
        uri.path().parts().clone(),
        uri.fragment().cloned(),
      )),
      (s, Some(un), None, None) => Some((
        Address::from((s.to_string(), un.to_string())),
        uri.path().parts().clone(),
        uri.fragment().cloned(),
      )),
      _ => None,
    }
  }

  pub fn from_uri(uri: Uri) -> Self {
    let (address, elems, fragment) = Self::address_from_uri(uri).unwrap();
    Self::of_root(address, "/".to_string()).with_children(elems, fragment)
  }

  pub fn from_string(s: &str) -> Self {
    let (address, elems, fragment) = Self::address_from_uri_string(s).unwrap();
    Self::of_root(address, "/".to_string()).with_children(elems, fragment)
  }

  pub fn of_child(parent: ActorPath, name: String, uid: u32) -> Self {
    if name.chars().any(|c| c == '/') {
      panic!(
        "/ is a path separator and is not legal in ActorPath names: [{}]",
        name
      )
    }
    if name.chars().any(|c| c == '#') {
      panic!(
        "# is a fragment separator and is not legal in ActorPath names: [{}]",
        name
      )
    }
    ActorPath::Child {
      parent: Arc::new(parent),
      name,
      uid,
    }
  }

  pub fn of_root(address: Address, name: String) -> Self {
    if !(name.len() == 1 || name[1..].chars().any(|c| c == '/')) {
      panic!("/ may only exist at the beginning of the root actors name, it is a path separator and is not legal in ActorPath names: [{}]", name)
    }
    if name.chars().any(|c| c == '#') {
      panic!(
        "# is a fragment separator and is not legal in ActorPath names: [{}]",
        name
      )
    }
    ActorPath::Root { address, name }
  }

  pub fn parent(&self) -> &ActorPath {
    match self {
      ActorPath::Root { .. } => self,
      ActorPath::Child { parent, .. } => parent,
    }
  }

  pub fn address(&self) -> &Address {
    let mut current_actor_path = self;
    loop {
      if let ActorPath::Root { address, .. } = current_actor_path {
        return address;
      }
      current_actor_path = current_actor_path.parent();
    }
  }

  pub fn name(&self) -> &str {
    match self {
      ActorPath::Root { name, .. } => name,
      ActorPath::Child { name, .. } => name,
    }
  }

  pub fn name_with_uid(&self) -> String {
    format!(
      "{}{}",
      self.name(),
      if self.uid() == 0 {
        "".to_string()
      } else {
        format!("#{}", self.uid())
      }
    )
  }

  pub fn uid(&self) -> u32 {
    match self {
      ActorPath::Root { .. } => 0,
      ActorPath::Child { uid, .. } => *uid,
    }
  }

  pub fn with_child(self, child: String, fragment: Option<String>) -> Self {
    let uid = fragment
      .map(|s| u32::from_str(&s).unwrap_or(0))
      .unwrap_or(0);
    ActorPath::of_child(self, child, uid)
  }

  pub fn with_children(self, child: Vec<String>, fragment: Option<String>) -> Self {
    if let Some((last, elements)) = child.split_last() {
      let new_self = elements.into_iter().fold(self, |path, elem| {
        if elem.is_empty() {
          path
        } else {
          path.with_child(elem.clone(), None)
        }
      });
      if last.is_empty() {
        new_self
      } else {
        new_self.with_child(last.clone(), fragment)
      }
    } else {
      let elem = child.first().cloned().unwrap();
      if elem.is_empty() {
        self
      } else {
        self.with_child(elem, fragment)
      }
    }
  }

  pub fn elements(&self) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_actor_path = self;
    while let ActorPath::Child { name, .. } = current_actor_path {
      result.push(name.clone());
      current_actor_path = current_actor_path.parent();
    }
    result.into_iter().rev().collect::<Vec<_>>()
  }

  pub fn root(&self) -> &ActorPath {
    let mut current_actor_path = self;
    while let ActorPath::Child { .. } = current_actor_path {
      current_actor_path = current_actor_path.parent();
    }
    current_actor_path
  }

  pub fn with_uid(self, uid: u32) -> Self {
    match self {
      ActorPath::Root { .. } => {
        if uid == 0 {
          self
        } else {
          panic!("")
        }
      }
      ActorPath::Child { parent, name, .. } => ActorPath::Child { parent, name, uid },
    }
  }

  pub fn is_root(&self) -> bool {
    match self {
      ActorPath::Root { .. } => true,
      _ => false,
    }
  }

  pub fn is_child(&self) -> bool {
    !self.is_root()
  }
}

#[test]
fn test_from_string() {
  let uri = "tcp://my_system@host:8080/test1/test2/test3#1";
  let actor_path = ActorPath::from_string(uri);
  println!("actor_path = {}", actor_path);
  assert_eq!(actor_path.to_string(), uri);
  assert!(!actor_path.is_root());
  assert!(actor_path.is_child());
  assert!(actor_path.root().is_root());
  assert!(!actor_path.root().is_child());
  assert_eq!(actor_path.root().parent(), actor_path.root());
  assert_eq!(actor_path.address(), actor_path.root().address());
  assert!(!actor_path.elements().is_empty());
}

#[test]
fn test_cmp_equal() {
  let uri = "tcp://my_system@host:8080/test1/test2/test3#1";
  let actor_path1 = ActorPath::from_string(uri);
  let actor_path2 = ActorPath::from_string(uri);
  assert_eq!(actor_path1, actor_path2);
}

#[test]
fn test_cmp_not_equal_1() {
  let uri1 = "tcp://my_system@host:8080/test1/test2/test3#1";
  let uri2 = "tcp://my_system@host:8080/test1/test2/test4#1";
  let actor_path1 = ActorPath::from_string(uri1);
  let actor_path2 = ActorPath::from_string(uri2);
  assert_ne!(actor_path1, actor_path2);
}

#[test]
fn test_cmp_not_equal_2() {
  let uri1 = "tcp://my_system@host:8080/test1/test2/test3#1";
  let uri2 = "tcp://my_system@host:8080/test1/test2/test3#2";
  let actor_path1 = ActorPath::from_string(uri1);
  let actor_path2 = ActorPath::from_string(uri2);
  assert_ne!(actor_path1, actor_path2);
}

#[test]
fn test_cmp_not_equal_3() {
  let uri1 = "tcp://my_system@host:8080/test1/test2/test3#1";
  let uri2 = "tcp://my_system@host:8080/test1/test2/test3#2";
  let actor_path1 = ActorPath::from_string(uri1);
  let actor_path2 = ActorPath::from_string(uri2);
  let mut hasher1 = DefaultHasher::default();
  let mut hasher2 = DefaultHasher::default();
  actor_path1.hash(&mut hasher1);
  actor_path2.hash(&mut hasher2);
  assert_ne!(hasher1.finish(), hasher2.finish());
}

