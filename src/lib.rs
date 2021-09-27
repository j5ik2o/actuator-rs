#![feature(associated_type_defaults)]
#![feature(option_result_contains)]

pub mod actor;
pub mod kernel;

#[macro_use]
extern crate once_cell;

#[cfg(test)]
extern crate env_logger as logger;
