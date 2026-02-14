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
mod type_registry;

use codegen::{cpp, rust, Codegen, CodegenCommands};

use log::Level;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    inputs: Vec<String>,
    #[command(subcommand)]
    command: CodegenCommands,
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8

}

fn main() -> Result<(), failure::Error> {
    let args = Args::parse();

    let verbose = match args.verbose {
        0 => Level::Info,
        1 => Level::Debug,
        _ => Level::Trace
    };

    simple_logger::init_with_level(verbose).unwrap();

    let mut registry = type_registry::TypeRegistry::new();
    let mut packets = Vec::new();

    for filename in args.inputs.iter().map(std::path::Path::new) {
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
        registry.collect_from_packet(&packet)?;
        packets.push(packet);
    }

    registry.compute_is_copy();

    if let CodegenCommands::RustCommand(ref rust_args) = args.command {
        rust::generate_shared(VERSION, &registry, rust_args)?;
    }

    for packet in packets {
        let mut generator: Box<dyn Codegen> = match &args.command {
            CodegenCommands::CppCommand(args) => Box::new(cpp::Generator::new(args)),
            CodegenCommands::RustCommand(args) => Box::new(rust::Generator::new(args, &registry)),
        };
        generator.generate(VERSION, &packet)?;
        info!("Generated packet {}", packet.type_());
    }
    Ok(())
}
