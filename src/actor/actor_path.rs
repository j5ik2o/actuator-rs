use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::actor::actor_cell::split_name_and_uid;
use oni_comb_uri_rs::models::uri::Uri;

use crate::actor::address::{actor_path_extractor, Address};

const VALID_SYMBOLS: &'static str = "-_.*$+:@&=,!~';";

const VALID_PATH_CODE: i32 = -1;
const EMPTY_PATH_CODE: i32 = -2;

fn is_valid_char(c: char) -> bool {
  matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' ) | VALID_SYMBOLS.chars().position(|p| p == c).is_some()
}

fn is_hex_char(c: char) -> bool {
  matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9')
}

fn validate(s: &str, pos: usize) -> i32 {
  let len = s.len();
  if pos < len {
    let c_vec: Vec<char> = s.chars().collect();
    match c_vec[pos] {
      char if is_valid_char(char) => validate(s, pos + 1),
      '%' if pos + 2 < len && is_hex_char(c_vec[pos + 1]) && is_hex_char(c_vec[pos + 2]) => validate(s, pos + 3),
      _ => pos as i32,
    }
  } else {
    VALID_PATH_CODE
  }
}

fn full_path_msg(full_path: String) -> String {
  if !full_path.is_empty() {
    format!(" (in path [{}])", full_path)
  } else {
    "".to_owned()
  }
}

pub fn validate_path_element(element: &str) {
  validate_path_element_with_full_path(element, "".to_string())
}

pub fn validate_path_element_with_full_path(element: &str, full_path: String) {
  match find_invalid_path_element_char_position(element) {
    VALID_PATH_CODE => {}
    EMPTY_PATH_CODE => {
      panic!("Actor path element must not be empty {}", full_path_msg(full_path));
    }
    invalidAt => {
      panic!(
        "Invalid actor path element [{}]{}, \
      illegal character [{}] \
      at position: {}. Actor paths MUST: not start with `$`,\
       include only ASCII letters and can only contain these special characters: \
       {}.",
        element,
        full_path_msg(full_path),
        element.chars().nth(invalidAt as usize).unwrap(),
        invalidAt,
        VALID_SYMBOLS
      );
    }
  }
}

pub fn is_valid_path_element(s: &str) -> bool {
  find_invalid_path_element_char_position(s) == VALID_PATH_CODE
}

fn find_invalid_path_element_char_position(s: &str) -> i32 {
  if s.is_empty() {
    EMPTY_PATH_CODE
  } else {
    let len = s.len();
    if len > 0 && s.chars().nth(0).unwrap() != '$' {
      validate(s, 0)
    } else {
      0
    }
  }
}

static EMPTY_ACTOR_PATH: Lazy<Vec<String>> = Lazy::new(|| vec!["".to_owned()]);

#[derive(Debug, Clone, Hash)]
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
    if (cur_me as *const ActorPath) == (cur_other as *const ActorPath) {
      return Some(Ordering::Equal);
    } else if cur_me.is_root() {
      return root_partial_cmp(cur_me, cur_other);
    } else if cur_other.is_root() {
      return match root_partial_cmp(cur_other, cur_me) {
        Some(Ordering::Greater) => Some(Ordering::Less),
        Some(Ordering::Less) => Some(Ordering::Greater),
        x => x,
      };
    } else {
      let x = cur_me.name_with_uid().partial_cmp(&cur_other.name_with_uid());
      if x != Some(Ordering::Equal) {
        return x;
      }
      log::debug!("{:?}, {:?}", cur_me, cur_other);
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

pub trait ActorPathBehavior {
  fn address(&self) -> &Address;

  fn elements(&self) -> Vec<String>;

  fn is_root(&self) -> bool;

  fn is_child(&self) -> bool;

  fn name(&self) -> &str;

  fn name_with_uid(&self) -> String;

  fn parent(&self) -> &ActorPath;

  fn root(&self) -> &ActorPath;

  fn uid(&self) -> u32;

  fn with_uid(self, uid: u32) -> Self;

  fn with_child(self, child: &str) -> Self;

  fn with_children(self, children: Vec<String>) -> Self;

  fn to_string_without_address(&self) -> String;

  fn to_string_with_address(&self, addr: &Address) -> String;
}

impl ActorPathBehavior for ActorPath {
  fn address(&self) -> &Address {
    let mut current_actor_path = self;
    loop {
      if let ActorPath::Root { address, .. } = current_actor_path {
        return address;
      }
      current_actor_path = current_actor_path.parent();
    }
  }

  fn elements(&self) -> Vec<String> {
    match self {
      ActorPath::Root { .. } => EMPTY_ACTOR_PATH.clone(),
      ActorPath::Child { .. } => {
        let mut result = Vec::new();
        let mut current_actor_path = self;
        while let ActorPath::Child { name, .. } = current_actor_path {
          result.push(name.clone());
          current_actor_path = current_actor_path.parent();
        }
        result.into_iter().rev().collect::<Vec<_>>()
      }
    }
  }

  fn is_root(&self) -> bool {
    match self {
      ActorPath::Root { .. } => true,
      _ => false,
    }
  }

  fn is_child(&self) -> bool {
    !self.is_root()
  }

  fn name(&self) -> &str {
    match self {
      ActorPath::Root { name, .. } => name,
      ActorPath::Child { name, .. } => name,
    }
  }

  fn name_with_uid(&self) -> String {
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

  fn parent(&self) -> &ActorPath {
    match self {
      ActorPath::Root { .. } => self,
      ActorPath::Child { parent, .. } => parent,
    }
  }

  fn root(&self) -> &ActorPath {
    let mut current_actor_path = self;
    while let ActorPath::Child { .. } = current_actor_path {
      current_actor_path = current_actor_path.parent();
    }
    current_actor_path
  }

  fn uid(&self) -> u32 {
    match self {
      ActorPath::Root { .. } => 0,
      ActorPath::Child { uid, .. } => *uid,
    }
  }

  fn with_uid(self, uid: u32) -> Self {
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

  fn with_child(self, child: &str) -> Self {
    let (name, uid) = split_name_and_uid(&child);
    log::debug!("name = {}, uid = {}", name, uid);
    ActorPath::of_child(self, name.to_string(), uid)
  }

  fn with_children(self, children: Vec<String>) -> Self {
    children.into_iter().fold(
      self,
      |path, elem| {
        if elem.is_empty() {
          path
        } else {
          path.with_child(&elem)
        }
      },
    )
  }

  fn to_string_without_address(&self) -> String {
    format!("/{}", self.elements().join("/"))
  }

  fn to_string_with_address(&self, addr: &Address) -> String {
    match self {
      ActorPath::Root { .. } => {
        if self.address().host.is_some() {
          format!("{}{}", self.address().to_string(), self.name())
        } else {
          format!("{}{}", addr.to_string(), self.name())
        }
      }
      ActorPath::Child { .. } => self.build_to_string(|ap| ap.to_string_with_address(addr)),
    }
  }
}

impl ActorPath {
  fn address_string_length_diff(&self, address: &Address) -> usize {
    let r = self.root();
    if r.address().host.is_some() {
      0
    } else {
      address.to_string().len() - r.address().to_string().len()
    }
  }

  fn to_string_length(&self) -> usize {
    self.to_string_offset() + self.name().len()
  }

  fn to_string_offset(&self) -> usize {
    match self.parent() {
      ActorPath::Root { address, name } => address.to_string().len() + name.len(),
      c @ ActorPath::Child { .. } => c.to_string_length() + 1,
    }
  }

  fn build_to_string<F>(&self, root_str: F) -> String
  where
    F: Fn(&ActorPath) -> String, {
    fn rec<F>(p: &ActorPath, mut sb: Vec<String>, root_str: F) -> Vec<String>
    where
      F: Fn(&ActorPath) -> String, {
      match p {
        r @ ActorPath::Root { .. } => {
          let mut root_s = root_str(r);
          sb.push(root_s);
          sb
        }
        c @ ActorPath::Child { .. } => {
          let mut s = String::new();
          if c.parent().is_child() {
            s.push_str("/")
          }
          s.push_str(c.name());
          sb.push(s);
          rec(c.parent(), sb, root_str)
        }
      }
    }
    let mut sb: Vec<String> = Vec::new();
    let result = rec(self, sb, root_str);
    result.into_iter().rev().collect::<Vec<_>>().join("")
  }

  pub fn from_uri(uri: Uri) -> Self {
    let (address, children) = actor_path_extractor::unapply(&uri.to_string()).unwrap();
    Self::of_root(address).with_children(children)
  }

  pub fn from_string(s: &str) -> Self {
    let (address, children) = actor_path_extractor::unapply(s).unwrap();
    log::debug!("address = {:?}, children = {:?}", address, children);
    Self::of_root(address).with_children(children)
  }

  pub fn of_child(parent: ActorPath, name: String, uid: u32) -> Self {
    if name.chars().any(|c| c == '/') {
      panic!("/ is a path separator and is not legal in ActorPath names: [{}]", name)
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

  pub fn of_root(address: Address) -> Self {
    Self::of_root_with_name(address, "/".to_string())
  }

  pub fn of_root_with_name(address: Address, name: String) -> Self {
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
}

#[cfg(test)]
mod tests {
  use crate::actor::actor_path::{root_partial_cmp, validate_path_element, ActorPath, ActorPathBehavior};
  use crate::actor::address::Address;
  use std::{env, panic};

  #[ctor::ctor]
  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    // env::set_var("RUST_LOG", "trace");
    let _ = env_logger::try_init();
  }

  // support parsing its String rep
  #[test]
  fn test_1() {
    let addr = Address::new("actuator", "mysys");
    let path1 = ActorPath::of_root(addr).with_child("user");
    log::debug!("path1 = {}, {:?}", path1, path1);
    let path2 = ActorPath::from_string(&path1.to_string());
    log::debug!("path2 = {}, {:?}", path2, path2);
    assert_eq!(path2, path1)
  }

  // support parsing remote paths
  #[test]
  fn test_2() {
    let remote = "akka://my_sys@host:1234/some/ref";
    let path = ActorPath::from_string(remote);
    log::debug!("path = {}, {:?}", path, path);
    assert_eq!(path.to_string(), remote);
  }

  // throw exception upon malformed paths
  #[test]
  fn test_3() {
    let result = panic::catch_unwind(|| {
      ActorPath::from_string("");
    });
    assert!(result.is_err());
    let result = panic::catch_unwind(|| {
      ActorPath::from_string("://hallo");
    });
    assert!(result.is_err());
    let result = panic::catch_unwind(|| {
      ActorPath::from_string("s://dd@:12");
    });
    assert!(result.is_err());
    let result = panic::catch_unwind(|| {
      ActorPath::from_string("s://dd@h:hd");
    });
    assert!(result.is_err());
    let result = panic::catch_unwind(|| {
      ActorPath::from_string("a://l:1/b");
    });
    assert!(result.is_err());
  }

  // create correct toString
  #[test]
  fn test_4() {
    let a = Address::new("actuator", "mysys");
    assert_eq!(ActorPath::of_root(a.clone()).to_string(), "actuator://mysys/");
    assert_eq!(
      ActorPath::of_root(a.clone()).with_child("user").to_string(),
      "actuator://mysys/user"
    );
    assert_eq!(
      ActorPath::of_root(a.clone())
        .with_child("user")
        .with_child("foo")
        .to_string(),
      "actuator://mysys/user/foo"
    );
    assert_eq!(
      ActorPath::of_root(a.clone())
        .with_child("user")
        .with_child("foo")
        .with_child("bar")
        .to_string(),
      "actuator://mysys/user/foo/bar"
    );
  }

  // have correct path elements
  #[test]
  fn test_5() {
    let vec = ActorPath::of_root(Address::new("actuator", "mysys"))
      .with_child("user")
      .with_child("foo")
      .with_child("bar")
      .elements();
    assert_eq!(vec, vec!["user", "foo", "bar"]);
  }

  // create correct to_string_without_address
  #[test]
  fn test_6() {
    let a = Address::new("actuator", "mysys");
    assert_eq!(ActorPath::of_root(a.clone()).to_string_without_address(), "/");
    assert_eq!(
      ActorPath::of_root(a.clone())
        .with_child("user")
        .to_string_without_address(),
      "/user"
    );
    assert_eq!(
      ActorPath::of_root(a.clone())
        .with_child("user")
        .with_child("foo")
        .to_string_without_address(),
      "/user/foo"
    );
    assert_eq!(
      ActorPath::of_root(a.clone())
        .with_child("user")
        .with_child("foo")
        .with_child("bar")
        .to_string_without_address(),
      "/user/foo/bar"
    );
  }

  // validate path elements
  #[test]
  fn test_7() {
    let result = panic::catch_unwind(|| validate_path_element(""));
    assert!(result.is_err());
  }

  // create correct toStringWithAddress
  #[test]
  fn test_8() {
    let local = Address::new("actuator", "mysys");
    let a = Address::new_with_host_port("actuator", "mysys", "aaa", 2552);
    let b = Address::new_with_host_port("actuator", "mysys", "bb", 2552);
    let c = Address::new_with_host_port("actuator", "mysys", "cccc", 2552);

    let root = ActorPath::of_root(local);

    assert_eq!(root.to_string_with_address(&a), "actuator://mysys@aaa:2552/".to_owned());
    assert_eq!(
      root.clone().with_child("user").to_string_with_address(&a),
      "actuator://mysys@aaa:2552/user"
    );
    assert_eq!(
      root
        .clone()
        .with_child("user")
        .with_child("foo")
        .to_string_with_address(&a),
      "actuator://mysys@aaa:2552/user/foo"
    );

    assert_eq!(root.to_string_with_address(&b), "actuator://mysys@bb:2552/".to_owned());
    assert_eq!(
      root.clone().with_child("user").to_string_with_address(&b),
      "actuator://mysys@bb:2552/user"
    );
    assert_eq!(
      root
        .clone()
        .with_child("user")
        .with_child("foo")
        .to_string_with_address(&b),
      "actuator://mysys@bb:2552/user/foo"
    );

    assert_eq!(
      root.to_string_with_address(&c),
      "actuator://mysys@cccc:2552/".to_owned()
    );
    assert_eq!(
      root.clone().with_child("user").to_string_with_address(&c),
      "actuator://mysys@cccc:2552/user"
    );
    assert_eq!(
      root
        .clone()
        .with_child("user")
        .with_child("foo")
        .to_string_with_address(&c),
      "actuator://mysys@cccc:2552/user/foo"
    );

    let root_a = ActorPath::of_root(a);
    assert_eq!(root_a.to_string_with_address(&b), "actuator://mysys@aaa:2552/");
    assert_eq!(
      root_a.clone().with_child("user").to_string_with_address(&b),
      "actuator://mysys@aaa:2552/user"
    );
    assert_eq!(
      root_a.with_child("user").with_child("foo").to_string_with_address(&b),
      "actuator://mysys@aaa:2552/user/foo"
    );
  }
}
