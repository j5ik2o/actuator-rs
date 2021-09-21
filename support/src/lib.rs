#[cfg(test)]
extern crate env_logger as logger;
#[macro_use]
extern crate log;

mod collections;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
