extern crate packet_schema;
extern crate failure;

fn main() -> Result<(), failure::Error> {
    use std::fs::File;
    let file    = File::open("test.xml")?;
    let schema  = packet_schema::Reader::load_packet(file)?;
    println!("{:?}", schema);
    Ok(())
}
