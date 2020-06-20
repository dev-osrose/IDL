extern crate schema;
extern crate failure;
extern crate heck;
extern crate clap;
#[macro_use] extern crate log;
extern crate simple_logger;

mod flat_ast;
mod flatten;
mod writer;
mod graph_passes;
mod codegen;

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
                .index(1))
        .arg(Arg::with_name("output")
                .help("uses selected output directory for generated files")
                .short("o")
                .long("output")
                .value_name("FOLDER")
                .takes_value(true))
        .arg(Arg::with_name("generator")
                .help("Specify which language the schema should be generated for")
                .short("g")
                .long("generator")
                .possible_values(&["cpp", "rust"])
                .required(true))
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


    let output_dir = std::path::Path::new(matches.value_of("output").unwrap_or("./"));

    let filename = std::path::Path::new(matches.value_of("INPUT").unwrap());
    debug!("filename {:?}", filename);
    use std::fs::File;
    let file = File::open(filename)?;
    let packet = schema::Reader::load_packet(file)?;
    let packet = flatten::flatten(filename.parent().unwrap_or(std::path::Path::new("./")), &packet)?;
    trace!("packet {:?}", packet);
    let packet = graph_passes::run(packet)?;
    debug!("packet {:#?}", packet);
    let mut generator: Box<dyn codegen::Codegen> = match matches.value_of("generator").unwrap() {
        "cpp" => Box::new(codegen::cpp::Generator::new(output_dir, VERSION)),
        "rust" => Box::new(codegen::rust::Generator::new(output_dir, VERSION)),
        _ => unreachable!()
    };
    for file in generator.generate(&packet)? {
        debug!("wrote {}", file);
    }
    info!("Generated packet structure");
    Ok(())
}
