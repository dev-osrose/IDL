use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;
use std::collections::HashSet;

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
            if PacketContent::is_type(elem) {
                PacketContent::type_from_name(elem)
            } else {
                match elem {
                    PacketContent::Element(ref e) => {
                        match e.type_().as_ref() {
                            "int8_t" => Some("i8".to_string()),
                            "uint8_t" => Some("u8".to_string()),
                            "int16_t" => Some("i16".to_string()),
                            "uint16_t" => Some("u16".to_string()),
                            "int32_t" => Some("i32".to_string()),
                            "uint32_t" => Some("u32".to_string()),
                            "int64_t" => Some("i64".to_string()),
                            "uint64_t" => Some("u64".to_string()),
                            "char" => Some("u8".to_string()),
                            "float" => Some("f32".to_string()),
                            "double" => Some("f64".to_string()),
                            "std::string" => Some("String".to_string()),
                            _ => Some(e.type_().to_string())
                        }
                    },
                    _ => None
                }
            }
        }).collect::<::std::collections::HashSet<String>>();

        // Need to drop out the struct
        cg!(self, "pub struct {} {{", packet.class_name());
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => self.element(elem)?,
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

    fn element(&mut self, elem: &Element) -> Result<()> {
        self.doc(elem.doc())?;

        if let Some(bitset) = elem.bitset() {
            if bitset.start == 0 {
                cg!(self, "{}: [bool; {}],", bitset.name, bitset.size);
            }
            return Ok(());
        }

        let (type_, bits) = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            let type_ = match o {
                Unbounded => format!("Vec<{}>", elem.type_()),
                Num(n) => format!("[{}; {}]", elem.type_(), n)
            };
            (type_, "".to_string())
        } else {
            let bits = elem.bits().map_or_else(|| "".to_string(), |b| format!(" : {}", b));
            (elem.type_().to_owned(), bits)
        };
        let default = match elem.init() {
            self::ElementInitValue::Default(d) => " = ".to_string() + d,
            _ => "".to_string()
        };
        cg!(self, "{}: {}{}{},", elem.name(), type_, bits, default);
        Ok(())
    }
}

fn clean_base(base: &str) -> String {
    if base.contains("::") {
        base.split("::").skip(1).collect()
    } else {
        base.to_string()
    }
}
