use crate::actor::address::Address;

#[test]
fn test_new() {
  let addr = Address::from(("tcp".to_string(), "host".to_string()));
  println!("{}", addr.to_string());
}
