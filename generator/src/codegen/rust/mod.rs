use std::fs::File;
use std::path::PathBuf;
use codegen::Codegen;
use ::{flat_ast, writer};

mod codegen_source;

pub struct Generator {
    output: PathBuf
}

impl Generator {
    pub fn new(args: &RustArgs) -> Self {
        Self{
            output: args.output_folder.clone().into()
        }
    }
}

impl Codegen for Generator {
    fn generate(&mut self, version: &str, packet: &flat_ast::Packet) -> Result<(), failure::Error> {
        let source_output = File::create(self.output.to_str().unwrap().to_owned() + &format!("/{}.rs", packet.filename()))?;
        debug!("source {:?}", source_output);
        let mut writer = writer::Writer::new(source_output);
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer, version.to_string());
        codegen.generate(&packet)?;
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
#[command(name="rust")]
pub struct RustArgs {
    #[arg(long)]
    output_folder: String
}

#[cfg(test)]
mod tests {
    use crate::{flat_ast::Packet, writer::Writer};
    use super::{codegen_source};

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
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer, "0".to_string());
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