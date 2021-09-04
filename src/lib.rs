#[cfg(test)]
extern crate env_logger as logger;
#[macro_use]
extern crate log;

#[allow(dead_code)]
pub mod actor;
pub mod actor_system;
pub mod config;
pub mod kernel;
