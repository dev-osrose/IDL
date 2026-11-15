//! Rust backend.
//!
//! The generated code encodes every packet exactly as the C++ backend's code
//! does, so that Rust and C++ peers interoperate:
//!
//! * elements are written in declaration order with no padding, integers and
//!   floats little endian;
//! * strings are NUL terminated, except a string with a `<length>`, which is
//!   written as at most that many bytes and read as exactly that many;
//! * an enumeration is written as its base integer, and values without a name
//!   are kept rather than rejected;
//! * a list with `occursSize` is preceded by its element count; a list without
//!   one runs to the end of the packet; `occurs="N"` is exactly N elements;
//! * consecutive `bits` elements of a sequence are packed, least significant
//!   bit first, into the smallest whole number of bytes;
//! * a `<choice>` is a union, written as its widest plain unsigned integer
//!   member; the members are views onto parts of that integer;
//! * a `<choice selector="...">` is an enum with a variant per branch, the
//!   bytes of the variant's fields following those of the earlier selector,
//!   which is written as the value the variant is for (see `selected.rs`).
//!
//! The generated code does not depend on a serialization library. It calls a
//! small runtime that the crate using it provides, at `--runtime-path`:
//!
//! ```text
//! BString        byte string (bstr::BString)
//! trait Wire     fn write(&self, &mut Writer); fn read(&mut Reader) -> Result<Self, WireError>,
//!                implemented for the integers, f32, f64 and BString (NUL terminated)
//! Writer         put_bytes(&[u8]), put_bits(u64, bytes)
//! Reader         take(n), get_bits(bytes), read_to_end(), read_counted(n), read_array()
//! WireError      with fn unknown_case(what: &'static str, value: i64) -> WireError
//!                (only for selector choices without a default branch)
//! trait Packet   Wire + const TYPE: PacketType, const NAME: &'static str
//! PacketType     one associated constant per packet type name
//! ```
//!
//! Constants used as element counts are taken from `--consts-path`, as
//! `usize`.
//!
//! `<definitions>` files give a module each, named after them, with their
//! enumerations (like those of the packets) and their constants (`pub const`,
//! of the Rust type matching the C++ one).

mod context;
mod definitions;
mod encode;
mod files;
mod names;
mod prim;
mod selected;
mod simple;
mod structure;
mod union;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_roundtrip;
#[cfg(test)]
mod tests_selected;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use ::flat_ast::{ComplexType, ComplexTypeContent, Definitions, Packet, PacketContent};

use self::context::{Ctx, Def};

type Result<T> = std::result::Result<T, String>;

#[derive(clap::Args, Debug, Clone, Default)]
#[command(name = "rust")]
pub struct RustArgs {
    /// Directory the generated modules are written to.
    #[arg(long)]
    pub output_folder: String,

    /// Path of the generated shared types module, as the packet modules
    /// refer to it.
    #[arg(long, default_value = "crate::shared_types")]
    pub shared_types_path: String,

    /// Path of the module providing the wire runtime (see the module docs).
    #[arg(long, default_value = "crate::wire")]
    pub runtime_path: String,

    /// Path of the module holding the constants used as counts (any
    /// integer type; they are converted to `usize` where used).
    #[arg(long, default_value = "crate::consts")]
    pub consts_path: String,

    /// An enum defined outside the XML (`<element enum="..."/>`), as
    /// `<C++ type>=<Rust path>`. The Rust type must implement `Wire`.
    #[arg(long)]
    pub extern_type: Vec<String>,

    /// Also generate `Sample` impls (random instances, for tests) using the
    /// `Gen` and `Sample` of this module.
    #[arg(long)]
    pub samples: Option<String>,

    /// Cargo feature the `Sample` impls are compiled under.
    #[arg(long, default_value = "testgen")]
    pub samples_feature: String,
}

/// Generates the modules of all `packets`: one per packet, `shared_types.rs`
/// with the types they use, one per set of `definitions`, and `mod.rs` tying
/// them together.
pub fn generate(
    version: &str,
    packets: &[Packet],
    definitions: &[Definitions],
    args: &RustArgs,
) -> std::result::Result<(), failure::Error> {
    let files = generate_files(version, packets, definitions, args).map_err(failure::err_msg)?;
    let dir = Path::new(&args.output_folder);
    std::fs::create_dir_all(dir)?;
    for (name, content) in &files {
        std::fs::write(dir.join(name), content)?;
    }
    Ok(())
}

/// The generated files, by file name.
pub(crate) fn generate_files(
    version: &str,
    packets: &[Packet],
    definitions: &[Definitions],
    args: &RustArgs,
) -> Result<BTreeMap<String, String>> {
    let ctx = Ctx::new(args, packets)?;
    let mut files = BTreeMap::new();

    let mut shared = String::new();
    for name in &ctx.needed {
        let def = ctx.types[name];
        if let Def::Complex(c) = def {
            if c.anonymous() {
                return Err(format!(
                    "{}: anonymous complex types are not supported by the Rust backend; name the type",
                    name
                ));
            }
        }
        ctx.definition(&mut shared, def)?;
    }
    files.insert(
        "shared_types.rs".to_owned(),
        format!("{}{}", files::header(version, "the types the packets share", args, false), shared),
    );

    let mut modules: Vec<(String, String)> = Vec::new();
    for packet in packets {
        let context = packet.type_().clone();
        let mut body = files::header(version, packet.type_(), args, true);
        let elements: Vec<_> = packet
            .contents()
            .iter()
            .filter_map(|c| match c {
                PacketContent::Element(e) => Some(e.clone()),
                _ => None,
            })
            .collect();
        // The types of the packet's own fields are reachable from its module,
        // so that using a packet needs only its module.
        let mut own: Vec<&str> = elements.iter().filter_map(|e| ctx.lookup(e.type_()).map(|(name, _)| name)).collect();
        // So are those of the branches of its selector choices.
        for e in &elements {
            if let Some((_, Def::Complex(c))) = ctx.lookup(e.type_()) {
                if let ComplexTypeContent::Selected(selected) = c.content() {
                    for case in selected.cases() {
                        own.extend(case.elements().iter().filter_map(|e| ctx.lookup(e.type_()).map(|(name, _)| name)));
                    }
                }
            }
        }
        own.sort_unstable();
        own.dedup();
        match own.as_slice() {
            [] => {}
            [one] => writeln!(body, "pub use {}::{};\n", args.shared_types_path, one).unwrap(),
            many => writeln!(body, "pub use {}::{{{}}};\n", args.shared_types_path, many.join(", ")).unwrap(),
        }
        let class = packet.class_name();
        ctx.structure(&mut body, class, packet.doc(), &elements, &context)?;
        writeln!(body, "impl Packet for {} {{", class).unwrap();
        writeln!(body, "    const TYPE: PacketType = PacketType::{};", packet.type_()).unwrap();
        writeln!(body, "    const NAME: &'static str = \"{}\";", class).unwrap();
        writeln!(body, "}}").unwrap();
        if modules.iter().any(|(module, _)| module == packet.filename()) {
            return Err(format!("{}: two packets would be generated as {}.rs", context, packet.filename()));
        }
        modules.push((packet.filename().clone(), class.clone()));
        files.insert(format!("{}.rs", packet.filename()), body);
    }
    let mut separate = Vec::new();
    for d in definitions {
        let file = format!("{}.rs", d.name());
        if files.contains_key(&file) || separate.contains(d.name()) {
            return Err(format!("{}: another packet or definitions would be generated as {}", d.name(), file));
        }
        files.insert(file, ctx.definitions(version, d)?);
        separate.push(d.name().clone());
    }
    modules.sort();
    separate.sort();
    files.insert("mod.rs".to_owned(), files::index(version, &modules, &separate));
    Ok(files)
}

impl<'a> Ctx<'a> {
    fn sample_feature(&self) -> Option<&str> {
        self.args.samples.as_ref().map(|_| self.args.samples_feature.as_str())
    }

    fn definition(&self, out: &mut String, def: Def) -> Result<()> {
        match def {
            Def::Simple(simple) => self.simple_type(out, simple, simple.name()),
            Def::Complex(complex) => self.complex_type(out, complex),
        }
    }

    fn complex_type(&self, out: &mut String, complex: &ComplexType) -> Result<()> {
        let context = complex.name().as_str();
        match complex.content() {
            ComplexTypeContent::Seq(seq) => {
                if seq.occurs().is_some() || seq.size_occurs().is_some() {
                    return Err(format!("{}: a sequence with occurs is not supported", context));
                }
                self.structure(out, complex.name(), complex.doc(), seq.elements(), context)
            }
            ComplexTypeContent::Choice(choice) => {
                if choice.occurs().is_some() || choice.size_occurs().is_some() {
                    return Err(format!("{}: a choice with occurs is not supported", context));
                }
                self.union(out, complex.name(), complex.doc(), choice, context)
            }
            ComplexTypeContent::Selected(selected) => self.selected(out, complex.name(), complex.doc(), selected, context),
            ComplexTypeContent::Empty => self.structure(out, complex.name(), complex.doc(), &[], context),
        }
    }
}

/// Writes the XML documentation of something as a doc comment.
fn doc_comment(out: &mut String, indent: &str, doc: &Option<String>) {
    if let Some(doc) = doc {
        for line in doc.lines().map(str::trim).filter(|l| !l.is_empty()) {
            writeln!(out, "{}/// {}", indent, line).unwrap();
        }
    }
}
