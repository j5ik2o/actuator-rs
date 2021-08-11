mod kernel;
mod actor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use log::{debug, error, log_enabled, info, Level};

        env_logger::init();

        debug!("this is a debug {}", "message");
        error!("this is printed by default");
        assert_eq!(2 + 2, 4);
    }
}
