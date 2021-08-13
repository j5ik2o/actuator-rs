use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::actor::actor_path::ActorPath;

#[derive(Clone)]
pub struct ActorUri {
  pub name: Arc<str>,
  pub path: ActorPath,
  pub host: Arc<str>,
}

impl Default for ActorUri {
  fn default() -> Self {
    Self {
      name: Arc::from(""),
      path: ActorPath::default(),
      host: Arc::from(""),
    }
  }
}

impl ActorUri {
  pub fn new(name: Arc<str>, path: ActorPath, host: Arc<str>) -> Self {
    Self { name, path, host }
  }
}

impl PartialEq for ActorUri {
  fn eq(&self, other: &ActorUri) -> bool {
    self.path == other.path
  }
}

impl Eq for ActorUri {}

impl Hash for ActorUri {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.path.hash(state);
  }
}

impl fmt::Display for ActorUri {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.path)
  }
}

impl fmt::Debug for ActorUri {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}://{}", self.host, self.path)
  }
}
