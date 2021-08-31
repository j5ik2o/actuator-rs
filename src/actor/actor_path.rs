use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::actor::address::Address;
use std::fmt::{Display, Formatter};
use crate::actor::actor_cell::ActorCell;
use uri_rs::{Uri};

#[derive(Debug, Clone, PartialEq, Hash)]
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

impl Display for ActorPath {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl ActorPath {
  pub fn from_string(s: String) -> Self {
    let uri = Uri::parse(&s).unwrap();
    let scheme = uri.schema();
    let authority = uri.authority().unwrap();
    let user_name = authority
      .user_info()
      .map(|ui| ui.user_name())
      .unwrap_or("actor_system");
    let host_name = uri.authority().map(|a| a.host_name().to_string());
    let port = uri
      .authority()
      .map(|a| a.port().map(|n| n as u32).unwrap_or(1000));
    let address = Address {
      protocol: scheme.to_string(),
      system: user_name.to_string(),
      host: host_name,
      port,
    };
    Self::of_root(address, "/".to_string()).slash_childs(uri.path().parts().clone())
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
    match self {
      ActorPath::Root { address, .. } => address,
      ActorPath::Child { .. } => self.root().address(),
    }
  }

  pub fn slash_child(self, child: String) -> Self {
    let (child_name, uid) = ActorCell::split_name_and_uid(child);
    ActorPath::Child {
      parent: Arc::from(self),
      name: child_name,
      uid,
    }
  }

  pub fn slash_childs(self, child: Vec<String>) -> Self {
    child.into_iter().fold(self, |path, elem| {
      if elem.is_empty() {
        path
      } else {
        path.slash_child(elem)
      }
    })
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
}

#[test]
fn test_slash() {
  let s = ActorPath::from_string("tcp://host:8080/test".to_string());
  println!("{}", s);
}
