use super::Codegen;
use super::flat_ast::*;
use std::io::Result;
//use ::heck::*;

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

    fn generate_header(&mut self) -> Result<String> {
        let filename = self.output_dir.to_str().unwrap().to_owned() + &format!("{}.h", self.stem);
        let file = std::fs::File::create(filename.clone())?;
        self.writer = Some(::writer::Writer::new(file));
        Ok(filename)
    }

    fn generate_source(&mut self) -> Result<String> {
        let filename = self.output_dir.to_str().unwrap().to_owned() + &format!("{}.cpp", self.stem);
        let file = std::fs::File::create(filename.clone())?;
        self.writer = Some(::writer::Writer::new(file));
        Ok(filename)
    }
}

impl<'a> Codegen for Generator<'a> {
    fn generate(&mut self, packet: &Packet) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}