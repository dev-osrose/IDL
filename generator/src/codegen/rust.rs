use super::Codegen;
use super::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;

pub struct Generator<'a> {
    output_dir: &'a std::path::Path,
    writer: Option<::writer::Writer<std::fs::File>>,
    version: &'a str
}

impl<'a> Generator<'a> {
    pub fn new(output_dir: &'a std::path::Path, version: &'a str) -> Self {
        Self {
            output_dir,
            writer: None,
            version
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
}

impl<'a> Codegen for Generator<'a> {
    fn generate(&mut self, packet: &Packet) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}