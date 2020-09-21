extern crate schema;
extern crate failure;
extern crate heck;
extern crate clap;
#[macro_use] extern crate log;
extern crate simple_logger;

mod flat_ast;
mod flatten;
mod writer;
mod codegen;
mod graph_passes;

use codegen::codegen_header;
use codegen::codegen_source;

use clap::{App, Arg};
use log::Level;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Packet generator")
        .version(VERSION)
        .author("L3nn0x <dragon83.super@gmail.com>")
        .about("Generates packets for osiROSE-new")
        .arg(Arg::with_name("INPUT")
                .help("Sets the input xml file")
                .required(true)
                .multiple(true)
                .index(1))
        .arg(Arg::with_name("outputh")
                .help("uses selected output directory for header")
                .short("h")
                .long("outputh")
                .value_name("FOLDER")
                .takes_value(true))
        .arg(Arg::with_name("outputc")
                .help("uses selected output directory for source")
                .short("c")
                .long("outputc")
                .value_name("FOLDER")
                .takes_value(true))
        .arg(Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("sets the level of verbosity"))
        .get_matches();

    let verbose = match matches.occurrences_of("v") {
        0 => Level::Info,
        1 => Level::Debug,
        _ => Level::Trace
    };

    simple_logger::init_with_level(verbose).unwrap();


    let outputh_dir = std::path::Path::new(matches.value_of("outputh").unwrap_or("./"));
    let outputc_dir = std::path::Path::new(matches.value_of("outputc").unwrap_or("./"));

    for filename in matches.values_of("INPUT").unwrap().map(std::path::Path::new) {
        debug!("filename {:?}", filename);
        use std::fs::File;
        let file = File::open(filename)?;
        let packet = schema::Reader::load_packet(file)?;
        if packet.type_() == "tmp" {
            continue;
        }
        let packet = flatten::flatten(filename.parent().unwrap_or(std::path::Path::new("./")), &packet)?;
        trace!("packet {:?}", packet);
        let packet = graph_passes::run(packet)?;
        debug!("packet {:#?}", packet);
        let header_output = File::create(outputh_dir.to_str().unwrap().to_owned() + &format!("/{}.h", packet.filename()))?;
        debug!("header {:?}", header_output);
        let mut writer = writer::Writer::new(header_output);
        let mut codegen = codegen_header::CodeHeaderGenerator::new(&mut writer, VERSION.to_string());
        codegen.generate(&packet)?;
        let source_output = File::create(outputc_dir.to_str().unwrap().to_owned() + &format!("/{}.cpp", packet.filename()))?;
        debug!("source {:?}", source_output);
        let mut writer = writer::Writer::new(source_output);
        let mut codegen = codegen_source::CodeSourceGenerator::new(&mut writer);
        codegen.generate(&packet)?;
        info!("Generated packet {}", packet.type_());
    }
    Ok(())
}
