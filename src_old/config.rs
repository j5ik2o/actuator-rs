use std::collections::HashMap;
use toml::Value;
use std::any::Any;

pub struct Config<'a> {
  values: HashMap<String, &'a dyn Any>,
}

impl<'a> Config<'a> {
  pub fn new() -> Self {
    Self {
      values: HashMap::new(),
    }
  }

  pub fn set<T>(&mut self, k: String, v: &'a T)
  where
    T: Any,
  {
    self.values.insert(k, v);
  }

  pub fn get<T>(&self, key: String) -> Option<&T>
    where
    T: Any,
  {
    self.values.get(&key).and_then(|&v| v.downcast_ref::<T>())
  }

}

#[test]
fn testtoml() {
  let mut c = Config::new();
  let key = "aaa".to_string();
  let value = "bbb".to_string();
  c.set(key.clone(), &value);
  let s = c.get::<String>(key);
  println!("result = {:?}", s);

  let toml_str = r#"
        global_string = "test"
        global_integer = 5
        [server]
        ip = "127.0.0.1"
        port = 80
        [[peers]]
        ip = "127.0.0.1"
        port = 8080
        [[peers]]
        ip = "127.0.0.1"
    "#;

  let decoded: Value = toml::from_str(toml_str).unwrap();
  let server = decoded["server"]["ip"].as_str().unwrap();
  println!("{}", server);
  // let mut settings = config::Config::default();
  // settings
  //     // Add in `./Settings.toml`
  //     .merge(config::File::with_name("Settings")).unwrap()
  //     // Add in settings from the environment (with a prefix of APP)
  //     // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
  //     .merge(config::Environment::with_prefix("APP")).unwrap();
  //
  // println!("{:?}",
  //          settings.try_into::<HashMap<String, String>>().unwrap());
}
