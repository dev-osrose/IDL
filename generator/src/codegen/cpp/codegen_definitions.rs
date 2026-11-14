//! C++ for `<definitions>`: a header with the enumerations and constants in
//! namespace `RoseCommon`, and a source file that only includes it (so that
//! every XML file gives a header and a source file).

use std::io::{Result, Write};

use ::codegen::{const_value, ConstValue};
use ::flat_ast::*;

pub(crate) struct CodeDefinitionsGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>,
    version: String
}

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

impl<'a, W: Write> CodeDefinitionsGenerator<'a, W> {
    pub fn new(writer: &'a mut ::writer::Writer<W>, version: String) -> Self {
        Self { writer, version }
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.writer.write(val)?;
        Ok(self)
    }

    fn doc(&mut self, doc: &Option<String>) -> Result<()> {
        if let Some(doc) = doc {
            for line in doc.lines().map(str::trim).filter(|l| !l.is_empty()) {
                cg!(self, "// {}", line);
            }
        }
        Ok(())
    }

    pub fn generate(&mut self, definitions: &Definitions) -> Result<()> {
        cg!(self, "#pragma once\n");
        let version = self.version.clone();
        cg!(self, "/* Generated with IDL v{} */\n", version);
        if definitions.doc().is_some() {
            self.doc(definitions.doc())?;
            cg!(self);
        }
        cg!(self, "#include <cstdint>\n");
        cg!(self, "namespace RoseCommon {{");
        for content in definitions.contents() {
            match content {
                DefinitionsContent::Simple(simple) => self.simple(simple)?,
                DefinitionsContent::Constant(constant) => self.constant(constant)?
            }
        }
        cg!(self, "\n}}");
        Ok(())
    }

    fn simple(&mut self, simple: &SimpleType) -> Result<()> {
        cg!(self);
        self.doc(simple.doc())?;
        for content in simple.contents() {
            let SimpleTypeContent::Restriction(restrict) = content;
            let enumerators: Vec<&Enumeration> = restrict.contents().iter().filter_map(|c| match c {
                RestrictionContent::Enumeration(e) => Some(e),
                _ => None
            }).collect();
            if enumerators.is_empty() {
                return Err(invalid(format!("{}: only enumerations can be definitions", simple.xml_name())));
            }
            self.doc(restrict.doc())?;
            let kind = if simple.scoped() { "enum class" } else { "enum" };
            cg!(self, "{} {} : {} {{", kind, simple.xml_name(), restrict.base());
            self.writer.indent();
            for e in enumerators {
                self.doc(e.doc())?;
                cg!(self, "{} = {},", e.value(), e.id());
            }
            self.writer.dedent();
            cg!(self, "}};");
        }
        Ok(())
    }

    fn constant(&mut self, constant: &Constant) -> Result<()> {
        let value = match const_value(constant).map_err(invalid)? {
            ConstValue::Integer(_) => constant.value().trim().to_owned(),
            ConstValue::Float(v) if constant.type_().trim() == "float" => format!("{:?}f", v),
            ConstValue::Float(v) => format!("{:?}", v)
        };
        self.doc(constant.doc())?;
        cg!(self, "constexpr {} {} = {};", constant.type_().trim(), constant.name(), value);
        Ok(())
    }
}

/// The source file of `definitions`.
pub(crate) fn source(definitions: &Definitions) -> String {
    format!("#include \"{}.h\"\n", definitions.name())
}
