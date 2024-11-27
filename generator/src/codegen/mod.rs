use flat_ast;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! cg {
        ($v:expr) => (($v).write("")?);
        ($v:expr, $fmt:expr) => (($v).write(format!($fmt))?);
        ($v:expr, $fmt:expr, $($arg:tt)*) => (($v).write(format!($fmt, $($arg)*))?);
    }
}

// the codegen trait, implement this for your language
pub(crate) trait Codegen {
    fn generate(&mut self, version: &str, packet: &flat_ast::Packet) -> Result<(), failure::Error>;
}

pub mod cpp;
pub mod rust;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum CodegenCommands {
    #[command(name = "cpp")]
    CppCommand(cpp::CppArgs),
    #[command(name = "rust")]
    RustCommand(rust::RustArgs)
}