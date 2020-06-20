use super::flat_ast;
use std::io::Result;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! cg {
        ($v:expr) => (($v).write("")?);
        ($v:expr, $fmt:expr) => (($v).write(format!($fmt))?);
        ($v:expr, $fmt:expr, $($arg:tt)*) => (($v).write(format!($fmt, $($arg)*))?);
    }
}

pub mod cpp;
pub mod rust;

pub trait Codegen {
    fn generate(&mut self, packet: &flat_ast::Packet) -> Result<Vec<String>>;
}