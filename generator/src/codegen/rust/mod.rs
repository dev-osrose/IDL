use std::fs::File;
use std::path::PathBuf;
use codegen::Codegen;
use ::{flat_ast, writer};
use crate::type_registry::TypeRegistry;

mod codegen_source;

pub struct Generator<'a> {
    output: PathBuf,
    registry: &'a TypeRegistry,
    shared_types_path: String,
}

impl<'a> Generator<'a> {
    pub fn new(args: &RustArgs, registry: &'a TypeRegistry) -> Self {
        Self{
            output: args.output_folder.clone().into(),
            registry,
            shared_types_path: args.shared_types_path.clone(),
        }
    }
}

pub fn generate_shared(version: &str, registry: &TypeRegistry, args: &RustArgs) -> Result<(), failure::Error> {
    let output: PathBuf = args.output_folder.clone().into();
    let source_output = File::create(output.to_str().unwrap().to_owned() + "/shared_types.rs")?;
    let mut writer = writer::Writer::new(source_output);
    let mut codegen = codegen_source::CodeSourceGenerator::new_shared(&mut writer, version.to_string(), registry);
    codegen.generate_shared()?;
    Ok(())
}

impl<'a> Codegen for Generator<'a> {
    fn generate(&mut self, version: &str, packet: &flat_ast::Packet) -> Result<(), failure::Error> {
        let source_output = File::create(self.output.to_str().unwrap().to_owned() + &format!("/{}.rs", packet.filename()))?;
        debug!("source {:?}", source_output);
        let mut writer = writer::Writer::new(source_output);
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer, version.to_string(), self.registry, self.shared_types_path.clone());
        codegen.generate(&packet)?;
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
#[command(name="rust")]
pub struct RustArgs {
    #[arg(long)]
    output_folder: String,

    #[arg(long, default_value = "crate::shared_types")]
    shared_types_path: String,
}

#[cfg(test)]
mod tests {
    use type_registry::TypeRegistry;
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
        let registry = TypeRegistry::new();
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer, "0".to_string(), &registry, "crate::shared_types".to_string());
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