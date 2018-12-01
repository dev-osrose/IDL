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

use std::env;

fn main() -> Result<(), failure::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        println!("Error!");
        std::process::exit(1);
    } else if args.len() == 1 {
        println!("Usage: {} <file.xml (can be multiple files)>", args[0]);
        std::process::exit(1);
    }
    use std::fs::File;
    for filename in args[1..].iter() {
        let file = File::open(filename)?;
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
    }
    Ok(())
}
