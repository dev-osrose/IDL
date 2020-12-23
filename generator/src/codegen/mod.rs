#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! cg {
        ($v:expr) => (($v).write("")?);
        ($v:expr, $fmt:expr) => (($v).write(format!($fmt))?);
        ($v:expr, $fmt:expr, $($arg:tt)*) => (($v).write(format!($fmt, $($arg)*))?);
    }
}
pub mod codegen_header;
pub mod codegen_source;

#[cfg(test)]
mod tests {
    use crate::{flat_ast::Packet, writer::Writer};
    use super::{codegen_header};

    struct StringWriter {
        output: String
    }

    impl StringWriter {
        fn new() -> Self {
            Self { output: String::new() }
        }
    }

    impl std::io::Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output += std::str::from_utf8(buf).unwrap();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Into<String> for StringWriter {
        fn into(self) -> String {
            self.output
        }
    }

    fn call_header(packet: &Packet) -> std::io::Result<String> {
        let writer = StringWriter::new();
        let mut writer = Writer::new(writer);
        let mut codegen = codegen_header::CodeHeaderGenerator::new(&mut writer, "0".to_string());
        codegen.generate(packet)?;
        Ok(writer.into().into())
    }

    #[test]
    fn empty_packet() {
        let packet = Packet::new("PAKCS_PACKET".to_owned(), None);
        let result = call_header(&packet);
        assert!(result.is_ok());
    }
}