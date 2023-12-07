//! general tool.
//!
//! This module contains common tools for the program, such as: environment
//! variables, task generation macros.

mod env;
mod parser;
pub mod file;

pub use self::env::EnvVar;
pub use self::parser::{ParseError, Parser};
