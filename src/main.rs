extern crate packet_schema;
extern crate failure;

mod flat_ast;
mod flatten;
mod writer;

fn main() -> Result<(), failure::Error> {
    use std::fs::File;
    let file    = File::open("test.xml")?;
    let schema  = packet_schema::Reader::load_packet(file)?;
    let schema = flatten::flatten("./", &schema)?;
    println!("{:?}", schema);
    Ok(())
}
