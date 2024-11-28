use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;
use std::collections::HashMap;

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
        cg!(self, r#"use bincode::{{Encode, Decode}};"#);
        cg!(self, r#"use crate::packet::PacketPayload;"#);

        let mut iserialize: HashMap<String, String> = HashMap::new();
        iserialize.insert("int8_t".to_string(), "i8".to_string());
        iserialize.insert("uint8_t".to_string(), "u8".to_string());
        iserialize.insert("int16_t".to_string(), "i16".to_string());
        iserialize.insert("uint16_t".to_string(), "u16".to_string());
        iserialize.insert("int32_t".to_string(), "i32".to_string());
        iserialize.insert("uint32_t".to_string(), "u32".to_string());
        iserialize.insert("int64_t".to_string(), "i64".to_string());
        iserialize.insert("uint64_t".to_string(), "u64".to_string());
        iserialize.insert("char".to_string(), "u8".to_string());
        iserialize.insert("int".to_string(), "i32".to_string());
        iserialize.insert("unsigned int".to_string(), "u32".to_string());
        iserialize.insert("float".to_string(), "f32".to_string());
        iserialize.insert("double".to_string(), "f64".to_string());
        iserialize.insert("std::string".to_string(), "String".to_string());


        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(simple) => self.simple_type(simple, &iserialize)?,
                _ => {}
            }
        }
        cg!(self);
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(ref complex) => self.complex_type(complex, &iserialize)?,
                _ => {}
            };
        }

        cg!(self);

        cg!(self, r#"#[derive(Debug, Encode, Decode)]"#);
        cg!(self, "pub struct {} {{", packet.class_name().to_upper_camel_case());
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

        cg!(self);
        cg!(self, "impl PacketPayload for {} {{}}", packet.class_name().to_upper_camel_case());


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

    fn complex_type(&mut self, complex: &ComplexType, iserialize: &HashMap<String, String>) -> Result<()> {
        use ::flat_ast::ComplexTypeContent::*;
        if complex.inline() == false {

            // All unions need to be outside the struct
            match complex.content() {
                Choice(ref c) => {
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            cg!(self, "struct {} {{", elem.name());
                            self.indent();
                            for e in seq.elements() {
                                self.element(e, &iserialize)?;
                            }
                            self.dedent();
                            cg!(self, "}}");
                            cg!(self);
                        }
                    }

                    cg!(self, "#[repr(C)]");
                    cg!(self, r#"#[derive(Debug)]"#);
                    cg!(self, "union {}InternalData {{", complex.name());
                    self.indent();
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            cg!(self, "{}: {},", elem.name().to_snake_case(), elem.name());
                        } else {
                            self.element(elem, &iserialize)?;
                        }
                    }
                    self.dedent();
                    cg!(self, "}}");
                },
                _ => {}
            }

            cg!(self);
            cg!(self, r#"#[derive(Debug, Encode, Decode)]"#);
            cg!(self, "struct {} {{", complex.name());
            self.indent();
            match complex.content() {
                Seq(ref s) => {
                    for elem in s.elements() {
                        self.element(elem, &iserialize)?;
                    }
                },
                Choice(ref c) => {
                    cg!(self, "data: {}InternalData,", complex.name());
                },
                Empty => {}
            }
            self.dedent();
            cg!(self, "}}");
            cg!(self);
        }
        Ok(())
    }

    fn simple_type(&mut self, simple: &SimpleType, iserialize: &HashMap<String, String>) -> Result<()> {
        cg!(self);
        self.doc(simple.doc())?;
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name(), &iserialize)?
            }
        }
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

        let trimmed_type = elem.type_().trim().to_string();
        let rust_type = iserialize.get(elem.type_().trim()).unwrap_or_else(|| {
            debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
            &trimmed_type
        });

        let (type_, bits) = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            let type_ = match o {
                Unbounded => format!("Vec<{}>", rust_type),
                Num(n) => {
                    if n.parse::<usize>().is_ok() {
                        format!("[{}; {}]", rust_type, n)
                    } else {
                        format!("[{}; ({} as usize)]", rust_type, n)
                    }
                }
            };
            (type_, "".to_string())
        } else {
            let bits = elem.bits().map_or_else(|| "".to_string(), |b| format!(" : {}", b));
            (rust_type.to_owned().to_string(), bits)
        };
        // let default = match elem.init() {
        //     self::ElementInitValue::Default(d) => " = ".to_string() + d,
        //     _ => "".to_string()
        // };
        let name = rename_if_reserved(elem.name());
        // cg!(self, "{}: {}{}{},", elem.name(), type_, bits, default);
        cg!(self, "{}: {}{},", name, type_, bits);
        Ok(())
    }

    fn restrict(&mut self, restrict: &Restriction, name: &str, iserialize: &HashMap<String, String>) -> Result<()> {
        use self::RestrictionContent::*;
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();
        self.doc(restrict.doc())?;
        let base = restrict.base().trim().to_string();
        let rust_type = iserialize.get(restrict.base().trim()).unwrap_or_else(|| {
            debug!(r#"Type "{}" not found, outputting anyway"#, base);
            &base
        });

        if is_enum {
            cg!(self, r#"#[repr({})]"#, rust_type);
            cg!(self, r#"#[derive(Debug, Encode, Decode)]"#);
            cg!(self, "enum {} {{", name.to_upper_camel_case());
            self.indent();
            for content in restrict.contents() {
                if let Enumeration(en) = content {
                    self.doc(en.doc())?;
                    cg!(self, "{} = {},", en.value().to_upper_camel_case(), en.id());
                }
            }
        } else {
            cg!(self, r#"#[derive(Debug, Encode, Decode)]"#);
            cg!(self, "struct {} {{", name.to_upper_camel_case());
            self.indent();
            cg!(self, "{}: {},", name.to_string().to_snake_case(), rust_type);
        }

        self.dedent();
        cg!(self, "}}");
        Ok(())
    }
}

fn rename_if_reserved(name: &str) -> String {
    let reserved_keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
        "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
        "abstract", "become", "box", "do", "final", "macro", "override", "priv",
        "try", "typeof", "unsized", "virtual", "yield",
    ];

    if reserved_keywords.contains(&name) {
        format!("{}_", name.to_snake_case()) // Append a suffix to avoid conflicts
    } else {
        name.to_string().to_snake_case()
    }
}

