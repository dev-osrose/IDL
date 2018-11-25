extern crate packet_schema;
extern crate failure;
extern crate heck;

mod flat_ast;
mod flatten;
mod writer;
mod codegen;

fn main() -> Result<(), failure::Error> {
    use std::fs::File;
    let file = File::open("test.xml")?;
    let packet = packet_schema::Reader::load_packet(file)?;
    let packet = flatten::flatten("./", &packet)?;
    println!("{:?}", packet);
    let header_output = File::create(format!("{}.h", packet.filename()))?;
    let mut writer = writer::Writer::new(header_output);
    let mut codegen = codegen::CodeHeaderGenerator::new(&mut writer);
    codegen.generate(&packet)?;
    let source_output = File::create(format!("{}.cpp", packet.filename()))?;
    let mut writer = writer::Writer::new(source_output);
    let mut codegen = codegen::CodeSourceGenerator::new(&mut writer);
    codegen.generate(&packet)?;
    Ok(())
}
