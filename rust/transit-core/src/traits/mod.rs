//! Traits for parsing and adapting transit data.

mod adapter;
mod parser;

pub use adapter::Adapter;
pub use parser::{Parser, StreamingParser, Writer};
