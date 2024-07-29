#![doc = include_str!("../README.md")]

mod compiler;
mod error;
mod value;
mod identifier;
mod operation;
mod expression;
mod state;
mod state_machine;

#[cfg(test)]
mod tests;

pub use crate::error::{SML_Error, SML_Result};
pub use crate::state_machine::StateMachine;
