//! The code that writes, reads and samples a list of fields: those of a
//! struct, or of a variant of a selector choice. `access` gives the
//! expression for a field's value (`self.name` in a struct, `(*name)` in a
//! variant bound by reference); `pad` is the indentation of the statements.

use std::fmt::Write as _;

use super::names::mask;
use super::structure::{Field, Shape};

/// Total bytes of the bit run `run`.
fn run_bytes(runs: &[u32], run: usize) -> u32 {
    (runs[run] + 7) / 8
}

pub(super) fn write_fields(out: &mut String, fields: &[Field], runs: &[u32], pad: &str, access: &dyn Fn(&str) -> String) {
    // The last field of each bit run is where the run is written.
    let last_of_run = |index: usize, run: usize| match fields.get(index + 1).map(|f| &f.shape) {
        Some(Shape::Bits { run: next, .. }) => *next != run,
        _ => true,
    };
    for (index, field) in fields.iter().enumerate() {
        let f = access(&field.name);
        match &field.shape {
            Shape::Scalar => writeln!(out, "{}{}.write(w);", pad, f).unwrap(),
            Shape::Count { list } => writeln!(out, "{}({}.len() as {}).write(w);", pad, access(list), field.ty).unwrap(),
            Shape::Counted { .. } | Shape::Array => {
                writeln!(out, "{0}for item in &{1} {{\n{0}    item.write(w);\n{0}}}", pad, f).unwrap()
            }
            Shape::Masked { by, mask } => writeln!(
                out,
                "{0}for index in 0..(({1} as u64) & {2:#x}).count_ones() as usize {{\n{0}    {3}.get(index).cloned().unwrap_or_default().write(w);\n{0}}}",
                pad,
                access(by),
                mask,
                f
            )
            .unwrap(),
            Shape::List { count } => {
                if let Some(count) = count {
                    writeln!(out, "{}({}.len() as {}).write(w);", pad, f, count).unwrap();
                }
                writeln!(out, "{0}for item in &{1} {{\n{0}    item.write(w);\n{0}}}", pad, f).unwrap();
            }
            Shape::Bits { offset, width, run } => {
                if *offset == 0 {
                    writeln!(out, "{}let mut bits = 0u64;", pad).unwrap();
                }
                writeln!(out, "{}bits |= ({} as u64 & {:#x}) << {};", pad, f, mask(*width), offset).unwrap();
                if last_of_run(index, *run) {
                    writeln!(out, "{}w.put_bits(bits, {});", pad, run_bytes(runs, *run)).unwrap();
                }
            }
            Shape::Selector { body } => writeln!(out, "{}{}.selector().write(w);", pad, access(body)).unwrap(),
            Shape::Body { .. } => writeln!(out, "{}{}.write_case(w);", pad, f).unwrap(),
        }
    }
}

/// Reads every field into a local, named `local(field name)`.
pub(super) fn read_fields(out: &mut String, fields: &[Field], runs: &[u32], pad: &str, local: &dyn Fn(&str) -> String) {
    for field in fields {
        let f = &local(&field.name);
        match &field.shape {
            Shape::Scalar => writeln!(out, "{}let {} = Wire::read(r)?;", pad, f).unwrap(),
            Shape::Count { .. } | Shape::Selector { .. } => {
                writeln!(out, "{}let {}: {} = Wire::read(r)?;", pad, f, field.ty).unwrap()
            }
            Shape::Masked { by, mask } => writeln!(
                out,
                "{}let {} = r.read_counted((({} as u64) & {:#x}).count_ones() as usize)?;",
                pad, f, local(by), mask
            )
            .unwrap(),
            // A negative count is no items.
            Shape::Counted { by } => {
                writeln!(out, "{}let {} = r.read_counted(usize::try_from({}).unwrap_or(0))?;", pad, f, local(by)).unwrap()
            }
            Shape::List { count: Some(count) } => {
                writeln!(out, "{}let count = <{} as Wire>::read(r)? as usize;", pad, count).unwrap();
                writeln!(out, "{}let {} = r.read_counted(count)?;", pad, f).unwrap();
            }
            Shape::List { count: None } => writeln!(out, "{}let {} = r.read_to_end();", pad, f).unwrap(),
            Shape::Array => writeln!(out, "{}let {} = r.read_array()?;", pad, f).unwrap(),
            Shape::Bits { offset, width, run } => {
                if *offset == 0 {
                    writeln!(out, "{}let bits = r.get_bits({})?;", pad, run_bytes(runs, *run)).unwrap();
                }
                writeln!(out, "{}let {} = ((bits >> {}) & {:#x}) as {};", pad, f, offset, mask(*width), field.ty).unwrap();
            }
            Shape::Body { selector } => {
                writeln!(out, "{}let {} = {}::read_case({}, r)?;", pad, f, field.ty, local(selector)).unwrap()
            }
        }
    }
}

/// A random value for a field (not a selector: its choice decides it).
pub(super) fn sample_value(field: &Field) -> String {
    match &field.shape {
        Shape::Scalar | Shape::Array | Shape::Count { .. } | Shape::Body { .. } | Shape::Selector { .. } => {
            "Sample::sample(g)".into()
        }
        Shape::List { .. } | Shape::Counted { .. } | Shape::Masked { .. } => "g.list()".into(),
        Shape::Bits { width, .. } => format!("(g.next_u64() & {:#x}) as {}", mask(*width), field.ty),
    }
}

/// What makes a sample consistent: a count has to match its list to survive
/// a round trip; so does a list counted by the bits of a flag.
pub(super) fn sample_fixes(fields: &[Field], access: &dyn Fn(&str) -> String) -> Vec<String> {
    fields
        .iter()
        .filter_map(|field| match &field.shape {
            Shape::Count { list } => Some(format!("{} = {}.len() as {};", access(&field.name), access(list), field.ty)),
            Shape::Masked { by, mask } => Some(format!(
                "{} = (0..(({} as u64) & {:#x}).count_ones()).map(|_| Sample::sample(g)).collect();",
                access(&field.name),
                access(by),
                mask
            )),
            _ => None,
        })
        .collect()
}
