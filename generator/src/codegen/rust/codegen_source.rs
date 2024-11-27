use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;
use std::collections::HashMap;
use schema::ast::Occurs::Unbounded;

pub (crate) struct CodeSourceGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>,
    version: String
}

impl<'a, W: Write> CodeSourceGenerator<'a, W> {
    pub fn new(writer: &'a mut ::writer::Writer<W>, version: String) -> Self {
        Self {
            writer,
            version
        }
    }

    fn indent(&mut self) {
        self.writer.indent();
    }

    fn dedent(&mut self) {
        self.writer.dedent();
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.writer.write(val)?;
        Ok(self)
    }

    pub fn generate(&mut self, packet: &Packet) -> Result<()> {
        let version = self.version.clone();
        cg!(self, "/* Generated with IDL v{} */\n", version);
        cg!(self);
        cg!(self, r#"use bincode::{{Encode, Decode}};"#);
        cg!(self);


        cg!(self, r#"#[derive(Debug, Encode, Decode)]"#);

        let iserialize = packet.contents().iter().filter_map(|elem| {
            match elem {
                PacketContent::Element(ref e) => {
                    let rust_type = match e.type_().as_ref() {
                        "int8_t" => "i8",
                        "uint8_t" => "u8",
                        "int16_t" => "i16",
                        "uint16_t" => "u16",
                        "int32_t" => "i32",
                        "uint32_t" => "u32",
                        "int64_t" => "i64",
                        "uint64_t" => "u64",
                        "char" => "u8",
                        "float" => "f32",
                        "double" => "f64",
                        "std::string" => "String",
                        _ => e.type_().as_str(),
                    };
                    Some((e.type_().to_string(), rust_type.to_string())) // Map key and value
                }
                _ => None,
            }
        }).collect::<HashMap<String, String>>(); // Collect into a HashMap

        // Need to drop out the struct
        cg!(self, "pub struct {} {{", packet.class_name());
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => self.element(elem, &iserialize)?,
                _ => {}
            };
        }
        self.dedent();
        cg!(self, "}}");


        Ok(())
    }

    fn doc(&mut self, doc: &Option<String>) -> Result<()> {
        match doc {
            None => (),
            Some(doc) => {
                for line in doc.lines() {
                    match line.trim() {
                        "" => (),
                        line => {
                            cg!(self, "// {}", line);
                        }
                    }
                }
            }
        };
        Ok(())
    }

    fn element(&mut self, elem: &Element, iserialize: &HashMap<String, String>) -> Result<()> {
        self.doc(elem.doc())?;

        if let Some(bitset) = elem.bitset() {
            if bitset.start == 0 {
                cg!(self, "{}: [bool; {}],", bitset.name, bitset.size);
            }
            return Ok(());
        }

        let Some(rust_type) = iserialize.get(elem.type_()) else { warn!(r#"Type "{}" not found"#, elem.type_()); return Ok(()) };

        let (type_, bits) = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            let type_ = match o {
                Unbounded => format!("Vec<{}>", rust_type),
                Num(n) => format!("[{}; {}]", rust_type, n)
            };
            (type_, "".to_string())
        } else {
            let bits = elem.bits().map_or_else(|| "".to_string(), |b| format!(" : {}", b));
            (rust_type.to_owned(), bits)
        };
        let default = match elem.init() {
            self::ElementInitValue::Default(d) => " = ".to_string() + d,
            _ => "".to_string()
        };
        cg!(self, "{}: {}{}{},", elem.name(), type_, bits, default);
        Ok(())
    }
}
