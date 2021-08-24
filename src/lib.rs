#[allow(dead_code)]
pub mod actor;
mod actor_system;
mod kernel;

#[macro_use]
extern crate log;
#[cfg(test)]
extern crate env_logger as logger;
