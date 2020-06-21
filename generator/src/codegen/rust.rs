use super::Codegen;
use super::flat_ast::*;
use std::io::Result;
use ::heck::*;

pub struct Generator<'a> {
    output_dir: &'a std::path::Path,
    writer: Option<::writer::Writer<std::fs::File>>,
    version: &'a str,
    stem: &'a str
}

impl<'a> Generator<'a> {
    pub fn new(output_dir: &'a std::path::Path, stem: &'a str, version: &'a str) -> Self {
        Self {
            output_dir,
            writer: None,
            version,
            stem
        }
    }

    fn convert_basic_types(type_: &str) -> Option<&'static str> {
        match type_ {
            "char" => Some("char"),
            "u8" => Some("u8"),
            "i8" => Some("i8"),
            "u16" => Some("u16"),
            "i16" => Some("i16"),
            "u32" => Some("u32"),
            "i32" => Some("i32"),
            "u64" => Some("u64"),
            "i64" => Some("i64"),
            "float" => Some("f32"),
            "double" => Some("f64"),
            "string" => Some("String"),
            _ => None
        }
    }

    fn indent(&mut self) {
        self.writer.as_mut().unwrap().indent();
    }

    fn dedent(&mut self) {
        self.writer.as_mut().unwrap().dedent();
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.writer.as_mut().unwrap().write(val)?;
        Ok(self)
    }

    fn generate_file(&mut self) -> Result<String> {
        let filename = self.output_dir.to_str().unwrap().to_owned() + &format!("{}.rs", self.stem);
        let file = std::fs::File::create(filename.clone())?;
        self.writer = Some(::writer::Writer::new(file));
        Ok(filename)
    }

    fn write_choice(&mut self, name: &str, choice: &Choice) -> Result<()> {
        if let Some(doc) = choice.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "#[derive(Serialize, Deserialize)]");
        cg!(self, "pub enum {} {{", name);
        self.indent();
        for elem in choice.elements() {
            let type_ = if let Some(basic) = Generator::convert_basic_types(elem.type_()) {
                basic
            } else {
                elem.type_()
            };
            let type_ = if let Some(occurs) = elem.occurs().as_ref() {
                match occurs {
                    Occurs::Unbounded => format!("Vec<{}>", type_),
                    Occurs::Num(n) => format!("[{}; {}]", type_, n)
                }
            } else {
                type_.to_string()
            };
            if let Some(doc) = elem.doc().as_ref() {
                cg!(self, "/* {} */", doc);
            }
            cg!(self, "pub {}({}),", elem.name(), type_);
        }
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn write_sequence(&mut self, name: &str, s: &Sequence) -> Result<()> {
        if let Some(doc) = s.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "#[derive(Serialize, Deserialize)]");
        cg!(self, "pub struct {} {{", name);
        self.indent();
        for elem in s.elements() {
            let type_ = if let Some(basic) = Generator::convert_basic_types(elem.type_()) {
                basic
            } else {
                elem.type_()
            };
            let type_ = if let Some(occurs) = elem.occurs().as_ref() {
                match occurs {
                    Occurs::Unbounded => format!("Vec<{}>", type_),
                    Occurs::Num(n) => format!("[{}; {}]", type_, n)
                }
            } else {
                type_.to_string()
            };
            if let Some(doc) = elem.doc().as_ref() {
                cg!(self, "/* {} */", doc);
            }
            cg!(self, "pub {}: {},", elem.name(), type_);
        }
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn write_restriction(&mut self, name: &str, r: &Restriction) -> Result<()> {
        if let Some(doc) = r.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "#[derive(Serialize, Deserialize)]");
        cg!(self, "pub enum {} {{", name);
        self.indent();
        for elem in r.contents() {
            match elem {
                RestrictionContent::Enumeration(e) => {
                    if let Some(doc) = e.doc().as_ref() {
                        cg!(self, "/* {} */", doc);
                    }
                    cg!(self, "pub {} = {},", e.value().to_shouty_snake_case(), e.id());
                }
            }
        }
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn write_simple_type(&mut self, s: &SimpleType) -> Result<()> {
        if let Some(doc) = s.doc() {
            cg!(self, "/* {} */", doc);
        }
        match s.content() {
            SimpleTypeContent::Restriction(r) => self.write_restriction(s.name(), r)?,
        }
        Ok(())
    }

    fn write_complex_type(&mut self, c: &ComplexType) -> Result<()> {
        if let Some(doc) = c.doc() {
            cg!(self, "/* {} */", doc);
        }
        match c.content() {
            ComplexTypeContent::Choice(choice) => self.write_choice(c.name(), choice)?,
            ComplexTypeContent::Seq(s) => self.write_sequence(c.name(), s)?,
            ComplexTypeContent::Empty => {
                cg!(self, "#[derive(Serialize, Deserialize)]");
                cg!(self, "pub struct {};", c.name());
            }
        }
        Ok(())
    }

    fn write_declarations(&mut self, packet: &Packet) -> Result<()> {
        let mut choice = Choice::new(packet.doc().as_ref().cloned());
        for elem in packet.contents().iter().filter_map(|e| match e {
            PacketContent::Element(e) => Some(e),
            _ => None
        }) {
            choice.add_element(elem.clone());
        }
        self.write_choice("Packet", &choice)?;
        for type_ in packet.contents().iter().filter(|e| e.is_type()) {
            cg!(self);
            match type_ {
                PacketContent::Simple(s) => self.write_simple_type(s)?,
                PacketContent::Complex(c) => self.write_complex_type(c)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn write_definitions(&mut self, packet: &Packet) -> Result<()> {
        Ok(())
    }
}

impl<'a> Codegen for Generator<'a> {
    fn generate(&mut self, packet: &Packet) -> Result<Vec<String>> {
        let filename = self.generate_file()?;

        cg!(self, "/* Generated with IDL v{} */\n", self.version);
        cg!(self,"use crate::serde_derive::*;");
        cg!(self);
        cg!(self, "// --------------- DECLARATIONS -----------------");
        self.write_declarations(packet)?;
        cg!(self);
        cg!(self, "// --------------- DEFINITIONS ------------------");
        self.write_definitions(packet)?;
        
        Ok(vec![filename])
    }
}