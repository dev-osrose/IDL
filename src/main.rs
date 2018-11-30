extern crate packet_schema;
extern crate failure;
extern crate heck;

mod flat_ast;
mod flatten;
mod writer;
#[macro_use]
mod codegen_header;
mod codegen_source;
mod graph_passes;

fn main() -> Result<(), failure::Error> {
    use std::fs::File;
    let file = File::open("srv_login_req.xml")?;
    let packet = packet_schema::Reader::load_packet(file)?;
    let packet = flatten::flatten("./", &packet)?;
    let packet = graph_passes::run(packet)?;
    println!("{:?}", packet);
    let header_output = File::create(format!("{}.h", packet.filename()))?;
    let mut writer = writer::Writer::new(header_output);
    let mut codegen = codegen_header::CodeHeaderGenerator::new(&mut writer);
    codegen.generate(&packet)?;
    let source_output = File::create(format!("{}.cpp", packet.filename()))?;
    let mut writer = writer::Writer::new(source_output);
    let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer);
    codegen.generate(&packet)?;
    Ok(())
}
