//#![feature(try_trait)]

//! This is documentation for the rust_forth module
//!
//!

pub use error::ForthError;
pub use forth_compiler::Token;

pub mod error;
pub mod forth_compiler;
pub mod stack_machine;

pub enum Handled {
    Handled,
    NotHandled,
}
