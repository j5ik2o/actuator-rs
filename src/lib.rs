#[allow(dead_code)]
pub mod actor;
pub mod actor_system;
pub mod kernel;
pub mod config;

#[macro_use]
extern crate log;
#[cfg(test)]
extern crate env_logger as logger;
