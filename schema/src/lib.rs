#[macro_use] extern crate failure;
extern crate xml;

pub mod ast;
mod error;
mod parse;
mod reader;
mod attributes;

pub use error::ParseError;
pub use reader::Reader;