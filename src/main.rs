extern crate packet_schema;
extern crate failure;
extern crate heck;

mod flat_ast;
mod flatten;
mod writer;
mod codegen;

fn main() -> Result<(), failure::Error> {
    use std::fs::File;
    let file    = File::open("test.xml")?;
    let schema  = packet_schema::Reader::load_packet(file)?;
    let schema = flatten::flatten("./", &schema)?;
    println!("{:?}", schema);
    let header_output = File::create("test.h")?;
    let mut writer = writer::Writer::new(header_output);
    let mut header_codegen = codegen::CodeHeaderGenerator::new(&mut writer);
    header_codegen.generate(&schema)?;
    Ok(())
}
