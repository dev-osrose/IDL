//! Structs: packets and `<sequence>` complex types.

use std::fmt::Write as _;

use ::flat_ast::{Element, ElementInitValue, Occurs};

use super::context::Ctx;
use super::names::{check_ident, field_name, mask, parse_int};
use super::prim::prim;
use super::{doc_comment, Result};

/// How one element of a struct is stored and encoded.
struct Field {
    name: String,
    /// Full Rust type of the struct field.
    ty: String,
    shape: Shape,
    default: String,
    doc: Option<String>,
}

enum Shape {
    Scalar,
    /// `Vec<T>`, with the Rust type of the count written before it, if any.
    List { count: Option<&'static str> },
    /// `Vec<T>` whose count is the value of the earlier field `by`
    /// (`occursRef`).
    Counted { by: String },
    /// An integer that holds how many items field `list` has: read as it
    /// comes, written as the list's length.
    Count { list: String },
    /// `Vec<T>` with as many items as the earlier field `by` has bits set
    /// under `mask` (`occursBits`, `occursMask`); written as exactly that
    /// many.
    Masked { by: String, mask: u64 },
    Array,
    /// Part of a run of bit fields: offset and width within the run.
    Bits { offset: u32, width: u32, run: usize },
}

/// Default of a plain element: the XML default when it is a number, otherwise
/// the type's own default.
fn scalar_default(element: &Element) -> String {
    let numeric = prim(element.type_()).filter(|p| p.rust != "BString");
    let text = match element.init() {
        ElementInitValue::Default(text) => Some(text.trim()),
        _ => None,
    };
    match (numeric, text) {
        (Some(p), Some(text))
            if text.parse::<i64>().is_ok() || parse_int(text).is_some() || text.parse::<f64>().is_ok() =>
        {
            let literal = text.trim_end_matches(['f', 'F']);
            format!("{} as {}", literal, p.rust)
        }
        _ => "Default::default()".into(),
    }
}

impl<'a> Ctx<'a> {
    fn fields(&self, elements: &[Element], context: &str) -> Result<(Vec<Field>, Vec<u32>)> {
        let mut fields: Vec<Field> = Vec::new();
        // Total width of each run of bit fields.
        let mut runs: Vec<u32> = Vec::new();
        let mut in_run = false;

        for (index, element) in elements.iter().enumerate() {
            let name = field_name(element.name());
            if fields.iter().any(|f| f.name == name) {
                return Err(format!("{}: two elements are named {}", context, name));
            }
            let context = format!("{}: element {}", context, element.name());
            let base = self.type_path(element, &context)?;
            let doc = element.doc().clone();

            if let Some(width) = element.bits() {
                let p = prim(element.type_())
                    .filter(|p| p.integer)
                    .ok_or(format!("{}: bit field of a non-integer type", context))?;
                if element.occurs().is_some() || width == 0 || width > p.bits {
                    return Err(format!("{}: unsupported bit field", context));
                }
                if !in_run {
                    runs.push(0);
                    in_run = true;
                }
                let run = runs.len() - 1;
                let offset = runs[run];
                runs[run] += width;
                if runs[run] > 64 {
                    return Err(format!("{}: run of bit fields is wider than 64 bits", context));
                }
                fields.push(Field {
                    name,
                    ty: base,
                    shape: Shape::Bits { offset, width, run },
                    default: "0".into(),
                    doc,
                });
                continue;
            }
            in_run = false;

            if let Some((by, _)) = element.occurs_bits() {
                if !matches!(element.occurs(), Some(Occurs::Unbounded))
                    || element.size_occurs().is_some()
                    || element.occurs_ref().is_some()
                {
                    return Err(format!(
                        "{}: occursBits needs occurs=\"unbounded\", no occursSize and no occursRef",
                        context
                    ));
                }
                let mask = element.occurs_mask().unwrap().map_err(|e| format!("{}: {}", context, e))?;
                let by = field_name(by);
                elements[..index]
                    .iter()
                    .position(|e| field_name(e.name()) == by)
                    .filter(|at| {
                        let e = &elements[*at];
                        e.occurs().is_none() && e.bits().is_none() && prim(e.type_()).is_some_and(|p| p.integer)
                    })
                    .ok_or(format!(
                        "{}: occursBits {:?} is not an earlier integer element",
                        context, by
                    ))?;
                fields.push(Field {
                    name,
                    ty: format!("Vec<{}>", base),
                    shape: Shape::Masked { by, mask },
                    default: "Vec::new()".into(),
                    doc,
                });
                continue;
            }

            if let Some(by) = element.occurs_ref() {
                if !matches!(element.occurs(), Some(Occurs::Unbounded)) || element.size_occurs().is_some() {
                    return Err(format!(
                        "{}: occursRef needs occurs=\"unbounded\" and no occursSize",
                        context
                    ));
                }
                let by = field_name(by);
                let count = elements[..index]
                    .iter()
                    .position(|e| field_name(e.name()) == by)
                    .filter(|at| {
                        let e = &elements[*at];
                        e.occurs().is_none() && e.bits().is_none() && prim(e.type_()).is_some_and(|p| p.integer)
                    })
                    .ok_or(format!(
                        "{}: occursRef {:?} is not an earlier integer element",
                        context, by
                    ))?;
                // Bit fields come out one field per element, so the index
                // into `elements` is the index into `fields` as well.
                let counter = &mut fields[count];
                if !matches!(counter.shape, Shape::Scalar) {
                    return Err(format!("{}: {} already counts another list", context, by));
                }
                counter.shape = Shape::Count { list: name.clone() };
                fields.push(Field {
                    name,
                    ty: format!("Vec<{}>", base),
                    shape: Shape::Counted { by },
                    default: "Vec::new()".into(),
                    doc,
                });
                continue;
            }

            let field = match element.occurs() {
                None => {
                    if element.size_occurs().is_some() {
                        return Err(format!("{}: occursSize without occurs", context));
                    }
                    Field {
                        name,
                        default: scalar_default(element),
                        ty: base,
                        shape: Shape::Scalar,
                        doc,
                    }
                }
                Some(Occurs::Unbounded) => {
                    let count = match element.size_occurs() {
                        Some(size) => Some(
                            prim(size)
                                .filter(|p| p.integer && p.unsigned)
                                .ok_or(format!(
                                    "{}: occursSize {:?} is not an unsigned integer",
                                    context, size
                                ))?
                                .rust,
                        ),
                        None => {
                            // Without a count the list runs to the end of the
                            // packet: nothing after it could be read back.
                            if index + 1 != elements.len() {
                                return Err(format!(
                                    "{}: a list without occursSize must be the last element",
                                    context
                                ));
                            }
                            None
                        }
                    };
                    Field {
                        name,
                        ty: format!("Vec<{}>", base),
                        shape: Shape::List { count },
                        default: "Vec::new()".into(),
                        doc,
                    }
                }
                Some(occurs @ Occurs::Num(_)) => {
                    if element.size_occurs().is_some() {
                        return Err(format!(
                            "{}: occursSize on a fixed size array is not supported",
                            context
                        ));
                    }
                    let n = self.count(occurs, &context)?;
                    Field {
                        name,
                        ty: format!("[{}; {}]", base, n),
                        shape: Shape::Array,
                        default: "std::array::from_fn(|_| Default::default())".into(),
                        doc,
                    }
                }
            };
            fields.push(field);
        }
        Ok((fields, runs))
    }

    /// A struct for a packet or a `<sequence>`, with its encoding.
    pub(super) fn structure(
        &self,
        out: &mut String,
        name: &str,
        doc: &Option<String>,
        elements: &[Element],
        context: &str,
    ) -> Result<()> {
        check_ident(name, context)?;
        let (fields, runs) = self.fields(elements, context)?;
        let run_bytes = |run: usize| (runs[run] + 7) / 8;
        // The last field of each bit run is where the run is written.
        let last_of_run = |index: usize, run: usize| match fields.get(index + 1).map(|f| &f.shape) {
            Some(Shape::Bits { run: next, .. }) => *next != run,
            _ => true,
        };

        doc_comment(out, "", doc);
        writeln!(out, "#[derive(Debug, Clone, PartialEq)]").unwrap();
        writeln!(out, "pub struct {} {{", name).unwrap();
        for field in &fields {
            doc_comment(out, "    ", &field.doc);
            writeln!(out, "    pub {}: {},", field.name, field.ty).unwrap();
        }
        writeln!(out, "}}\n").unwrap();

        writeln!(out, "impl Default for {} {{\n    fn default() -> Self {{\n        Self {{", name).unwrap();
        for field in &fields {
            writeln!(out, "            {}: {},", field.name, field.default).unwrap();
        }
        writeln!(out, "        }}\n    }}\n}}\n").unwrap();

        writeln!(out, "impl Wire for {} {{\n    fn write(&self, w: &mut Writer) {{", name).unwrap();
        for (index, field) in fields.iter().enumerate() {
            let f = &field.name;
            match &field.shape {
                Shape::Scalar => writeln!(out, "        self.{}.write(w);", f).unwrap(),
                Shape::Count { list } => {
                    writeln!(out, "        (self.{}.len() as {}).write(w);", list, field.ty).unwrap()
                }
                Shape::Counted { .. } => {
                    writeln!(out, "        for item in &self.{} {{\n            item.write(w);\n        }}", f).unwrap()
                }
                Shape::Masked { by, mask } => writeln!(
                    out,
                    "        for index in 0..((self.{} as u64) & {:#x}).count_ones() as usize {{\n            self.{}.get(index).cloned().unwrap_or_default().write(w);\n        }}",
                    by, mask, f
                )
                .unwrap(),
                Shape::List { count } => {
                    if let Some(count) = count {
                        writeln!(out, "        (self.{}.len() as {}).write(w);", f, count).unwrap();
                    }
                    writeln!(out, "        for item in &self.{} {{\n            item.write(w);\n        }}", f).unwrap();
                }
                Shape::Array => {
                    writeln!(out, "        for item in &self.{} {{\n            item.write(w);\n        }}", f).unwrap()
                }
                Shape::Bits { offset, width, run } => {
                    if *offset == 0 {
                        writeln!(out, "        let mut bits = 0u64;").unwrap();
                    }
                    writeln!(out, "        bits |= (self.{} as u64 & {:#x}) << {};", f, mask(*width), offset).unwrap();
                    if last_of_run(index, *run) {
                        writeln!(out, "        w.put_bits(bits, {});", run_bytes(*run)).unwrap();
                    }
                }
            }
        }
        writeln!(out, "    }}\n").unwrap();

        writeln!(out, "    fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {{").unwrap();
        for field in &fields {
            let f = &field.name;
            match &field.shape {
                Shape::Scalar => writeln!(out, "        let {} = Wire::read(r)?;", f).unwrap(),
                Shape::Count { .. } => writeln!(out, "        let {}: {} = Wire::read(r)?;", f, field.ty).unwrap(),
                Shape::Masked { by, mask } => writeln!(
                    out,
                    "        let {} = r.read_counted((({} as u64) & {:#x}).count_ones() as usize)?;",
                    f, by, mask
                )
                .unwrap(),
                Shape::Counted { by } => {
                    // A negative count is no items.
                    writeln!(out, "        let {} = r.read_counted(usize::try_from({}).unwrap_or(0))?;", f, by).unwrap()
                }
                Shape::List { count: Some(count) } => {
                    writeln!(out, "        let count = <{} as Wire>::read(r)? as usize;", count).unwrap();
                    writeln!(out, "        let {} = r.read_counted(count)?;", f).unwrap();
                }
                Shape::List { count: None } => writeln!(out, "        let {} = r.read_to_end();", f).unwrap(),
                Shape::Array => writeln!(out, "        let {} = r.read_array()?;", f).unwrap(),
                Shape::Bits { offset, width, run } => {
                    if *offset == 0 {
                        writeln!(out, "        let bits = r.get_bits({})?;", run_bytes(*run)).unwrap();
                    }
                    writeln!(out, "        let {} = ((bits >> {}) & {:#x}) as {};", f, offset, mask(*width), field.ty)
                        .unwrap();
                }
            }
        }
        let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
        writeln!(out, "        Ok(Self {{ {} }})\n    }}\n}}\n", names.join(", ")).unwrap();

        if let Some(feature) = self.sample_feature() {
            // A count has to match its list to survive a round trip; so does
            // a list counted by the bits of a flag.
            let fixes: Vec<String> = fields
                .iter()
                .filter_map(|field| match &field.shape {
                    Shape::Count { list } => {
                        Some(format!("sample.{} = sample.{}.len() as {};", field.name, list, field.ty))
                    }
                    Shape::Masked { by, mask } => Some(format!(
                        "sample.{} = (0..((sample.{} as u64) & {:#x}).count_ones()).map(|_| Sample::sample(g)).collect();",
                        field.name, by, mask
                    )),
                    _ => None,
                })
                .collect();
            let open = if fixes.is_empty() { "Self {" } else { "let mut sample = Self {" };
            writeln!(
                out,
                "#[cfg(feature = \"{}\")]\nimpl Sample for {} {{\n    fn sample(g: &mut Gen) -> Self {{\n        {}",
                feature, name, open
            )
            .unwrap();
            for field in &fields {
                let f = &field.name;
                match &field.shape {
                    Shape::Scalar | Shape::Array | Shape::Count { .. } => {
                        writeln!(out, "            {}: Sample::sample(g),", f).unwrap()
                    }
                    Shape::List { .. } | Shape::Counted { .. } | Shape::Masked { .. } => {
                        writeln!(out, "            {}: g.list(),", f).unwrap()
                    }
                    Shape::Bits { width, .. } => {
                        writeln!(out, "            {}: (g.next_u64() & {:#x}) as {},", f, mask(*width), field.ty).unwrap()
                    }
                }
            }
            if fixes.is_empty() {
                writeln!(out, "        }}\n    }}\n}}\n").unwrap();
            } else {
                writeln!(out, "        }};").unwrap();
                for fix in &fixes {
                    writeln!(out, "        {}", fix).unwrap();
                }
                writeln!(out, "        sample\n    }}\n}}\n").unwrap();
            }
        }
        Ok(())
    }
}
