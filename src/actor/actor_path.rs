use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::fmt;

pub struct ActorPath(Arc<str>);

impl ActorPath {
  pub fn new(path: &str) -> Self {
    ActorPath(Arc::from(path))
  }
}

impl PartialEq for ActorPath {
  fn eq(&self, other: &ActorPath) -> bool {
    self.0 == other.0
  }
}

impl PartialEq<str> for ActorPath {
  fn eq(&self, other: &str) -> bool {
    *self.0 == *other
  }
}

impl Eq for ActorPath {}

impl Hash for ActorPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl fmt::Display for ActorPath {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl fmt::Debug for ActorPath {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl Clone for ActorPath {
  fn clone(&self) -> Self {
    ActorPath(self.0.clone())
  }
}
