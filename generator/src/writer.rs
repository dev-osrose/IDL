use std::io::{Result, Write};

pub struct Writer<T> where T: Write {
    writer: T,
    indent: i32
}

impl<T: Write> Writer<T> {
    pub fn new(writer: T) -> Self {
        Writer {
            writer: writer,
            indent: 0
        }
    }

    fn pad(&mut self) -> Result<&mut Self> {
        for _ in 0..self.indent {
            self.writer.write(b"    ")?;
        }
        Ok(self)
    }

    pub fn get_indent(&self) -> i32 {
        self.indent
    }

    pub fn indent(&mut self) -> &mut Self {
        self.indent += 1;
        self
    }

    pub fn dedent(&mut self) -> &mut Self {
        self.indent -= 1;
        self
    }

    pub fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.pad()?.writer.write_fmt(format_args!("{}\n", val.as_ref()))?;
        Ok(self)
    }
}