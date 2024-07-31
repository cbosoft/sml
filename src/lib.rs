#![doc = include_str!("../README.md")]

mod compiler;
mod error;
pub mod examples;
mod value;
mod identifier;
mod operation;
mod expression;
mod state;
mod state_machine;
mod refcount;

pub use crate::error::{SML_Error, SML_Result};
pub use crate::state_machine::StateMachine;
pub use crate::compiler::compile;
