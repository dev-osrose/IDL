extern crate packet_schema;
extern crate failure;
extern crate heck;
extern crate clap;

mod flat_ast;
mod flatten;
mod writer;
#[macro_use]
mod codegen_header;
mod codegen_source;
mod graph_passes;

use clap::{App, Arg};

use std::env;

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Packet generator")
        .version("0.1")
        .author("L3nn0x <dragon83.super@gmail.com>")
        .about("Generates packets for osiROSE-new")
        .arg(Arg::with_name("INPUT")
                .help("Sets the input xml file")
                .required(true)
                .index(1))
        .arg(Arg::with_name("output")
                .help("uses selected output directory")
                .short("o")
                .long("output")
                .value_name("FOLDER")
                .takes_value(true))
        .arg(Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("sets the level of verbosity"))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    let verbose = match matches.occurrences_of("v") {
        0 => "info",
        1 => "debug",
        2 | _ => "trace",
    };

    let output_dir = matches.value_of("output").unwrap_or("./");

    use std::fs::File;
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
    Ok(())
}
