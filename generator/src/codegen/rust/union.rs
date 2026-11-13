//! Unions: `<choice>` complex types.
//!
//! The C++ generator turns a choice into a union and writes it as its widest
//! plain unsigned integer member. Here that integer is the storage, and every
//! member, including the fields of an inline `<sequence>`, is a getter and a
//! setter over part of it.

use std::fmt::Write as _;

use ::flat_ast::{Choice, Element};

use super::context::{restriction, Ctx, Def};
use super::names::{check_ident, field_name, mask};
use super::prim::{prim, Prim};
use super::{doc_comment, Result};

/// One member as seen through the storage integer.
struct View {
    accessor: String,
    ty: String,
    /// Expression turning `bits` into the member's type.
    get: String,
    /// Expression turning `value` into bits.
    set: String,
    offset: u32,
    width: u32,
}

impl<'a> Ctx<'a> {
    pub(super) fn union(
        &self,
        out: &mut String,
        name: &str,
        doc: &Option<String>,
        choice: &Choice,
        context: &str,
    ) -> Result<()> {
        check_ident(name, context)?;
        // The widest plain unsigned integer member, the last one winning a tie,
        // as in the C++ generator.
        let mut raw: Option<Prim> = None;
        for e in choice.elements() {
            if choice.inline_seqs().contains_key(e.name()) {
                continue;
            }
            if let Some(p) = prim(e.type_()).filter(|p| p.integer && p.unsigned && e.bits().is_none() && e.occurs().is_none()) {
                if raw.as_ref().map_or(true, |current| p.bits >= current.bits) {
                    raw = Some(p);
                }
            }
        }
        let raw = raw.ok_or(format!(
            "{}: a choice needs a plain unsigned integer member to be encodable",
            context
        ))?;

        let mut views: Vec<View> = Vec::new();
        for e in choice.elements() {
            match choice.inline_seqs().get(e.name()) {
                Some(seq) => {
                    let mut offset = 0;
                    for field in seq.elements() {
                        offset = self.view(field, offset, &raw, &mut views, context)?;
                    }
                }
                None => {
                    self.view(e, 0, &raw, &mut views, context)?;
                }
            }
        }

        let storage = raw.rust;
        doc_comment(out, "", doc);
        writeln!(out, "/// Union stored as one `{}`; the accessors are views onto parts of it.", storage).unwrap();
        writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]").unwrap();
        writeln!(out, "pub struct {}(pub {});\n\nimpl {} {{", name, storage, name).unwrap();
        for v in &views {
            let mask = mask(v.width);
            let setter = v.accessor.trim_end_matches('_');
            writeln!(out, "    pub fn {}(&self) -> {} {{", v.accessor, v.ty).unwrap();
            writeln!(out, "        let bits = (self.0 >> {}) & ({:#x} as {});", v.offset, mask, storage).unwrap();
            writeln!(out, "        {}\n    }}\n", v.get).unwrap();
            writeln!(out, "    pub fn set_{}(&mut self, value: {}) {{", setter, v.ty).unwrap();
            writeln!(out, "        let mask = ({:#x} as {}) << {};", mask, storage, v.offset).unwrap();
            writeln!(
                out,
                "        self.0 = (self.0 & !mask) | ((({} as {}) << {}) & mask);",
                v.set, storage, v.offset
            )
            .unwrap();
            writeln!(out, "    }}\n").unwrap();
        }
        writeln!(out, "}}\n").unwrap();
        writeln!(out, "impl Wire for {} {{\n    fn write(&self, w: &mut Writer) {{\n        self.0.write(w);\n    }}\n", name).unwrap();
        writeln!(out, "    fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {{\n        Ok(Self(Wire::read(r)?))\n    }}\n}}\n").unwrap();
        if let Some(feature) = self.sample_feature() {
            writeln!(out, "#[cfg(feature = \"{}\")]\nimpl Sample for {} {{\n    fn sample(g: &mut Gen) -> Self {{\n        Self(Sample::sample(g))\n    }}\n}}\n", feature, name).unwrap();
        }
        Ok(())
    }

    /// Adds the view of `element` at `offset` bits; returns where the next
    /// field of an inline sequence starts.
    fn view(&self, element: &Element, offset: u32, raw: &Prim, views: &mut Vec<View>, context: &str) -> Result<u32> {
        let context = format!("{}: member {}", context, element.name());
        if element.occurs().is_some() {
            return Err(format!("{}: a list cannot be part of a union", context));
        }
        let accessor = field_name(element.name());
        if views.iter().any(|v| v.accessor == accessor) {
            return Err(format!("{}: two members are named {}", context, accessor));
        }
        let (ty, base) = match prim(element.type_()) {
            Some(p) if p.integer => (p.rust.to_owned(), None),
            Some(_) => return Err(format!("{}: only integers can be part of a union", context)),
            None => match self.lookup(element.type_()) {
                Some((_, Def::Simple(simple))) if restriction(simple).1.is_none() => {
                    let base = prim(&restriction(simple).0)
                        .filter(|p| p.integer)
                        .ok_or(format!("{}: unsupported base", context))?;
                    (self.type_path(element, &context)?, Some(base))
                }
                _ => {
                    return Err(format!(
                        "{}: only integers and enumerations can be part of a union",
                        context
                    ))
                }
            },
        };
        let storage_bits = base
            .as_ref()
            .map(|b| b.bits)
            .or(prim(element.type_()).map(|p| p.bits))
            .unwrap_or(0);
        let width = match element.bits() {
            Some(bits) => bits,
            None => {
                if offset % 8 != 0 {
                    return Err(format!("{}: member is not byte aligned", context));
                }
                storage_bits
            }
        };
        if width == 0 || width > storage_bits || offset + width > raw.bits {
            return Err(format!("{}: member does not fit in the union's {} bits", context, raw.bits));
        }
        let (get, set) = match &base {
            Some(base) => (format!("{}(bits as {})", ty, base.rust), "value.0".to_owned()),
            None => (format!("bits as {}", ty), "value".to_owned()),
        };
        views.push(View {
            accessor,
            ty,
            get,
            set,
            offset,
            width,
        });
        Ok(offset + width)
    }
}
