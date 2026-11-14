use std::fs::File;
use std::path::PathBuf;
use codegen::Codegen;
use ::{flat_ast, writer};

mod codegen_definitions;
mod codegen_header;
mod codegen_source;

pub struct Generator {
    output_header: PathBuf,
    output_source: PathBuf
}

impl Generator {
    pub fn new(args: &CppArgs) -> Self {
        Self{
            output_header: args.output_header_folder.clone().into(),
            output_source: args.output_source_folder.clone().into()
        }
    }
}

impl Generator {
    /// Writes `<name>.h` with the enumerations and constants of
    /// `definitions`, and a `<name>.cpp` that includes it.
    pub fn generate_definitions(&self, version: &str, definitions: &flat_ast::Definitions) -> Result<(), failure::Error> {
        let header = self.output_header.join(format!("{}.h", definitions.name()));
        let mut writer = writer::Writer::new(File::create(&header)?);
        codegen_definitions::CodeDefinitionsGenerator::new(&mut writer, version.to_string()).generate(definitions)?;
        std::fs::write(self.output_source.join(format!("{}.cpp", definitions.name())), codegen_definitions::source(definitions))?;
        Ok(())
    }
}

impl Codegen for Generator {
    fn generate(&mut self, version: &str, packet: &flat_ast::Packet) -> Result<(), failure::Error> {
        let header_output = File::create(self.output_header.to_str().unwrap().to_owned() + &format!("/{}.h", packet.filename()))?;
        debug!("header {:?}", header_output);
        let mut writer = writer::Writer::new(header_output);
        let mut codegen = codegen_header::CodeHeaderGenerator::new(&mut writer, version.to_string());
        codegen.generate(&packet)?;
        let source_output = File::create(self.output_source.to_str().unwrap().to_owned() + &format!("/{}.cpp", packet.filename()))?;
        debug!("source {:?}", source_output);
        let mut writer = writer::Writer::new(source_output);
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer);
        codegen.generate(&packet)?;
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
#[command(name="cpp")]
pub struct CppArgs {
    #[arg(long)]
    output_header_folder: String,
    #[arg(long)]
    output_source_folder: String
}

#[cfg(test)]
mod tests {
    use crate::{flat_ast::Packet, writer::Writer};
    use super::{codegen_definitions, codegen_header};

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

    #[test]
    fn definitions_are_a_header_of_enums_and_constants() {
        let xml = r#"<definitions name = "packet_type">
            <documentation>Packet type ids.</documentation>
            <simpleType name = "ePacketType" scoped = "true">
                <restriction base = "uint16_t">
                    <enumeration value = "PAKSTART" id = "0x700" />
                    <enumeration value = "PAKCS_ALIVE" id = "0x700"><documentation>still there</documentation></enumeration>
                    <enumeration value = "PAKSS_ERROR" />
                </restriction>
            </simpleType>
            <constant name = "MAX_STAT" type = "uint32_t" value = "300" />
            <constant name = "TAB_SIZE" type = "uint8_t" value = "0x1E" />
            <constant name = "DROP_RANGE" type = "float" value = "50" />
        </definitions>"#;
        let d = match ::schema::Reader::load_document(std::io::Cursor::new(xml.as_bytes().to_vec())).unwrap() {
            ::schema::ast::Document::Definitions(d) => ::flatten::flatten_definitions(&d),
            other => panic!("{:?}", other),
        };
        let mut writer = Writer::new(StringWriter::new());
        codegen_definitions::CodeDefinitionsGenerator::new(&mut writer, "0".to_string()).generate(&d).unwrap();
        let header: String = writer.into().into();
        let expected = [
            "#pragma once",
            "// Packet type ids.",
            "#include <cstdint>",
            "namespace RoseCommon {",
            "enum class ePacketType : uint16_t {",
            "    PAKSTART = 1792,",
            "    // still there",
            "    PAKCS_ALIVE = 1792,",
            "    PAKSS_ERROR = 1793,",
            "};",
            "constexpr uint32_t MAX_STAT = 300;",
            "constexpr uint8_t TAB_SIZE = 0x1E;",
            "constexpr float DROP_RANGE = 50.0f;",
            "}",
        ];
        let mut rest = header.as_str();
        for line in expected {
            let at = rest.find(line).unwrap_or_else(|| panic!("{:?} not found in order in:\n{}", line, header));
            rest = &rest[at + line.len()..];
        }
        assert_eq!(codegen_definitions::source(&d), "#include \"packet_type.h\"\n");
    }
}
