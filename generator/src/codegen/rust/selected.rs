//! Selector choices: `<choice selector="...">`.
//!
//! A choice is an enum with a variant per branch. The variant decides the
//! value written for the selector (`selector()`), so the two cannot disagree;
//! the selector is no field of the struct holding the choice. The default
//! branch, if there is one, is the variant `Other`, which keeps the value it
//! was read with: given the value of another branch, it is read back as that
//! branch.

use std::fmt::Write as _;

use ::flat_ast::{Case, Selected};
use ::heck::ToUpperCamelCase;

use super::context::{enumerators, Ctx, Def};
use super::encode::{read_fields, sample_fixes, sample_value, write_fields};
use super::names::{check_ident, const_name, field_name};
use super::prim::prim;
use super::structure::{Field, Shape};
use super::{doc_comment, Result};

/// One branch as an enum variant.
struct Variant {
    name: String,
    /// The pattern matching its value of the selector; `None` for `Other`.
    pattern: Option<String>,
    fields: Vec<Field>,
    runs: Vec<u32>,
    doc: Option<String>,
}

/// The local a field of a variant is bound to: its name, unless that is a
/// name the generated code needs itself.
fn local(field: &str) -> String {
    match field {
        "w" | "r" | "g" => format!("{}_", field),
        _ => field.to_owned(),
    }
}

/// `name` or `name: local`, to bind or build a field.
fn bind(field: &str) -> String {
    let to = local(field);
    if to == field {
        to
    } else {
        format!("{}: {}", field, to)
    }
}

impl Variant {
    /// `Self::Name`, binding the fields by reference (`..` for the rest).
    fn binding(&self, skip: usize) -> String {
        let names: Vec<String> = self.fields[skip..].iter().map(|f| bind(&f.name)).collect();
        match (names.is_empty(), skip > 0) {
            (true, false) => format!("Self::{}", self.name),
            (true, true) => format!("Self::{} {{ .. }}", self.name),
            (false, false) => format!("Self::{} {{ {} }}", self.name, names.join(", ")),
            (false, true) => format!("Self::{} {{ {}, .. }}", self.name, names.join(", ")),
        }
    }

    /// `Self::Name { a, b }`, from locals of the fields' names.
    fn construct(&self) -> String {
        if self.fields.is_empty() {
            return format!("Self::{}", self.name);
        }
        let names: Vec<String> = self.fields.iter().map(|f| bind(&f.name)).collect();
        format!("Self::{} {{ {} }}", self.name, names.join(", "))
    }

    /// How many fields precede those of the branch: the selector, in `Other`.
    fn skip(&self) -> usize {
        if self.pattern.is_none() {
            1
        } else {
            0
        }
    }
}

impl<'a> Ctx<'a> {
    pub(super) fn selected(&self, out: &mut String, name: &str, doc: &Option<String>, selected: &Selected, context: &str) -> Result<()> {
        check_ident(name, context)?;
        let selector = field_name(selected.selector());
        let (ty, is_enum) = match prim(selected.selector_type()).filter(|p| p.integer) {
            Some(p) => (p.rust.to_owned(), false),
            None => match self.lookup(selected.selector_type()) {
                Some((ty, Def::Simple(simple))) if !enumerators(simple).is_empty() => (ty.to_owned(), true),
                _ => {
                    return Err(format!(
                        "{}: selector {} is neither an enumeration of the XML nor an integer",
                        context, selector
                    ))
                }
            },
        };

        let mut variants: Vec<Variant> = Vec::new();
        for case in selected.cases() {
            let variant = self.variant(case, &selector, &ty, is_enum, context)?;
            if variants.iter().any(|v| v.name == variant.name) {
                return Err(format!("{}: two branches would be the variant {}", context, variant.name));
            }
            variants.push(variant);
        }

        doc_comment(out, "", doc);
        writeln!(out, "/// What follows `{}`: the variant decides the value written for it.", selector).unwrap();
        writeln!(out, "#[derive(Debug, Clone, PartialEq)]\npub enum {} {{", name).unwrap();
        for v in &variants {
            doc_comment(out, "    ", &v.doc);
            if v.fields.is_empty() {
                writeln!(out, "    {},", v.name).unwrap();
                continue;
            }
            writeln!(out, "    {} {{", v.name).unwrap();
            for field in &v.fields {
                doc_comment(out, "        ", &field.doc);
                writeln!(out, "        {}: {},", field.name, field.ty).unwrap();
            }
            writeln!(out, "    }},").unwrap();
        }
        writeln!(out, "}}\n").unwrap();

        // The first named branch, failing that the default one.
        let first = variants.iter().find(|v| v.pattern.is_some()).or(variants.first());
        writeln!(out, "impl Default for {} {{\n    fn default() -> Self {{", name).unwrap();
        match first {
            None => return Err(format!("{}: a selector choice needs a branch", context)),
            Some(v) if v.fields.is_empty() => writeln!(out, "        Self::{}", v.name).unwrap(),
            Some(v) => {
                writeln!(out, "        Self::{} {{", v.name).unwrap();
                for field in &v.fields {
                    writeln!(out, "            {}: {},", field.name, field.default).unwrap();
                }
                writeln!(out, "        }}").unwrap();
            }
        }
        writeln!(out, "    }}\n}}\n").unwrap();

        writeln!(out, "impl {} {{", name).unwrap();
        writeln!(out, "    /// The value of `{}` this variant is written with.", selector).unwrap();
        writeln!(out, "    pub fn selector(&self) -> {} {{\n        match self {{", ty).unwrap();
        for v in &variants {
            match &v.pattern {
                Some(pattern) if v.fields.is_empty() => writeln!(out, "            Self::{} => {},", v.name, pattern).unwrap(),
                Some(pattern) => writeln!(out, "            Self::{} {{ .. }} => {},", v.name, pattern).unwrap(),
                None => writeln!(out, "            Self::{} {{ {}, .. }} => *{},", v.name, selector, selector).unwrap(),
            }
        }
        writeln!(out, "        }}\n    }}\n").unwrap();

        writeln!(out, "    /// Writes the fields of the variant; `{}` is written where it stands.", selector).unwrap();
        writeln!(out, "    pub fn write_case(&self, w: &mut Writer) {{\n        match self {{").unwrap();
        for v in &variants {
            let skip = v.skip();
            if v.fields.len() == skip {
                writeln!(out, "            {} => {{}}", v.binding(skip)).unwrap();
                continue;
            }
            writeln!(out, "            {} => {{", v.binding(skip)).unwrap();
            write_fields(out, &v.fields[skip..], &v.runs, "                ", &|f| format!("(*{})", local(f)));
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}\n    }}\n").unwrap();

        writeln!(out, "    /// Reads the variant for `selector`.").unwrap();
        writeln!(
            out,
            "    pub fn read_case(selector: {}, r: &mut Reader<'_>) -> Result<Self, WireError> {{\n        match selector {{",
            ty
        )
        .unwrap();
        for v in &variants {
            let arm = v.pattern.clone().unwrap_or_else(|| "_".to_owned());
            let skip = v.skip();
            if v.fields.len() == skip {
                let value = if skip == 1 && selector != "selector" {
                    format!("Self::{} {{ {}: selector }}", v.name, selector)
                } else {
                    v.construct()
                };
                writeln!(out, "            {} => Ok({}),", arm, value).unwrap();
                continue;
            }
            writeln!(out, "            {} => {{", arm).unwrap();
            if skip == 1 {
                writeln!(out, "                let {} = selector;", local(&selector)).unwrap();
            }
            read_fields(out, &v.fields[skip..], &v.runs, "                ", &local);
            writeln!(out, "                Ok({})\n            }}", v.construct()).unwrap();
        }
        if variants.iter().all(|v| v.pattern.is_some()) {
            let raw = if is_enum { "selector.0" } else { "selector" };
            writeln!(out, "            _ => Err(WireError::unknown_case(\"{}\", {} as i64)),", name, raw).unwrap();
        }
        writeln!(out, "        }}\n    }}\n}}\n").unwrap();

        if let Some(feature) = self.sample_feature() {
            self.sample_selected(out, feature, name, &ty, is_enum, &selector, &variants);
        }
        Ok(())
    }

    fn variant(&self, case: &Case, selector: &str, ty: &str, is_enum: bool, context: &str) -> Result<Variant> {
        let (name, pattern) = match case.value() {
            None => ("Other".to_owned(), None),
            Some(label) if is_enum => (label.to_upper_camel_case(), Some(format!("{}::{}", ty, const_name(label)))),
            Some(label) => {
                let value = ::selected::parse_int(label).ok_or(format!("{}: case {:?} is not an integer", context, label))?;
                let name = if value < 0 { format!("CaseMinus{}", -value) } else { format!("Case{}", value) };
                (name, Some(value.to_string()))
            }
        };
        check_ident(&name, context)?;
        let context = format!("{}: branch {}", context, name);
        let (mut fields, runs) = self.fields(case.elements(), &context)?;
        if pattern.is_none() {
            fields.insert(
                0,
                Field {
                    name: selector.to_owned(),
                    ty: ty.to_owned(),
                    shape: Shape::Scalar,
                    default: "Default::default()".into(),
                    doc: None,
                },
            );
        }
        Ok(Variant { name, pattern, fields, runs, doc: case.doc().clone() })
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_selected(&self, out: &mut String, feature: &str, name: &str, ty: &str, is_enum: bool, selector: &str, variants: &[Variant]) {
        writeln!(
            out,
            "#[cfg(feature = \"{}\")]\nimpl Sample for {} {{\n    fn sample(g: &mut Gen) -> Self {{\n        match g.below({}) {{",
            feature,
            name,
            variants.len()
        )
        .unwrap();
        for (index, v) in variants.iter().enumerate() {
            let arm = if index + 1 == variants.len() { "_".to_owned() } else { index.to_string() };
            if v.fields.is_empty() {
                writeln!(out, "            {} => {},", arm, v.construct()).unwrap();
                continue;
            }
            writeln!(out, "            {} => {{", arm).unwrap();
            let skip = v.skip();
            if skip == 1 {
                // Never the value of a named branch: it would read back as that one.
                let value = if is_enum {
                    format!("{}(g.next_u64() as {})", ty, self.enum_base(ty))
                } else {
                    format!("g.next_u64() as {}", ty)
                };
                let named: Vec<&str> = variants.iter().filter_map(|v| v.pattern.as_deref()).collect();
                if named.is_empty() {
                    writeln!(out, "                let mut {} = {};", local(selector), value).unwrap();
                } else {
                    writeln!(out, "                let mut {} = loop {{", local(selector)).unwrap();
                    writeln!(out, "                    let value = {};", value).unwrap();
                    writeln!(out, "                    if !matches!(value, {}) {{", named.join(" | ")).unwrap();
                    writeln!(out, "                        break value;\n                    }}\n                }};").unwrap();
                }
            }
            for field in &v.fields[skip..] {
                writeln!(out, "                let mut {} = {};", local(&field.name), sample_value(field)).unwrap();
            }
            for fix in sample_fixes(&v.fields, &local) {
                writeln!(out, "                {}", fix).unwrap();
            }
            writeln!(out, "                {}\n            }}", v.construct()).unwrap();
        }
        writeln!(out, "        }}\n    }}\n}}\n").unwrap();
    }

    /// The Rust integer an enumeration of the XML is stored as.
    fn enum_base(&self, ty: &str) -> &'static str {
        match self.lookup(ty) {
            Some((_, Def::Simple(simple))) => prim(&super::context::restriction(simple).0).map_or("u8", |p| p.rust),
            _ => "u8",
        }
    }
}
