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

/// The value of a `<constant>`, checked against its type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ConstValue {
    Integer(i128),
    Float(f64)
}

/// Reads the value of `constant`: an integer (decimal or `0x`) that fits its
/// integer type, or a number for `float` and `double`.
pub(crate) fn const_value(constant: &flat_ast::Constant) -> Result<ConstValue, String> {
    let (name, value) = (constant.name(), constant.value().trim());
    let range: (i128, i128) = match constant.type_().trim() {
        "float" | "double" => {
            return value
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(ConstValue::Float)
                .ok_or(format!("constant {}: {:?} is not a number", name, value));
        }
        "uint8_t" | "char" => (0, u8::MAX as i128),
        "int8_t" => (i8::MIN as i128, i8::MAX as i128),
        "uint16_t" => (0, u16::MAX as i128),
        "int16_t" => (i16::MIN as i128, i16::MAX as i128),
        "uint32_t" | "unsigned int" => (0, u32::MAX as i128),
        "int32_t" | "int" => (i32::MIN as i128, i32::MAX as i128),
        "uint64_t" => (0, u64::MAX as i128),
        "int64_t" => (i64::MIN as i128, i64::MAX as i128),
        other => return Err(format!("constant {}: type {:?} is not an integer, float or double", name, other))
    };
    let (negative, digits) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value)
    };
    let parsed = match digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
        Some(hex) => i128::from_str_radix(hex, 16),
        None => digits.parse::<i128>()
    }
    .map_err(|_| format!("constant {}: {:?} is not an integer", name, value))?;
    let parsed = if negative { -parsed } else { parsed };
    if parsed < range.0 || parsed > range.1 {
        return Err(format!("constant {}: {} does not fit in {}", name, value, constant.type_()));
    }
    Ok(ConstValue::Integer(parsed))
}

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum CodegenCommands {
    #[command(name = "cpp")]
    CppCommand(cpp::CppArgs),
    #[command(name = "rust")]
    RustCommand(rust::RustArgs)
}