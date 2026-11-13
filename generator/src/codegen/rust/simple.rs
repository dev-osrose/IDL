//! Simple types: enumerations and fixed size strings.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ::flat_ast::SimpleType;

use super::context::{enumerators, restriction, Ctx};
use super::names::{check_ident, const_name};
use super::prim::prim;
use super::{doc_comment, Result};

impl<'a> Ctx<'a> {
    pub(super) fn simple_type(&self, out: &mut String, simple: &SimpleType, context: &str) -> Result<()> {
        let name = simple.name();
        check_ident(name, context)?;
        let (base, length) = restriction(simple);
        let enumerators = enumerators(simple);
        doc_comment(out, "", simple.doc());

        if let Some(length) = length {
            if base != "std::string" || !enumerators.is_empty() {
                return Err(format!("{}: <length> is only supported on a plain std::string", context));
            }
            writeln!(out, "/// String of exactly {} bytes on the wire, with no terminator.", length).unwrap();
            writeln!(out, "#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub struct {}(pub BString);\n", name).unwrap();
            writeln!(out, "impl {} {{\n    pub const LEN: usize = {};\n}}\n", name, length).unwrap();
            writeln!(out, "impl Wire for {} {{\n    fn write(&self, w: &mut Writer) {{", name).unwrap();
            writeln!(out, "        // Shorter values are not padded: this is what the C++ writer does.").unwrap();
            writeln!(out, "        w.put_bytes(&self.0[..self.0.len().min(Self::LEN)]);\n    }}\n").unwrap();
            writeln!(out, "    fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {{").unwrap();
            writeln!(out, "        Ok(Self(BString::from(r.take(Self::LEN)?)))\n    }}\n}}\n").unwrap();
            if let Some(feature) = self.sample_feature() {
                writeln!(out, "#[cfg(feature = \"{}\")]\nimpl Sample for {} {{\n    fn sample(g: &mut Gen) -> Self {{\n        Self(g.letters(Self::LEN))\n    }}\n}}\n", feature, name).unwrap();
            }
            return Ok(());
        }

        let p = prim(&base)
            .filter(|p| p.integer)
            .ok_or(format!("{}: restriction base {:?} is not an integer", context, base))?;
        let storage = p.rust;
        writeln!(out, "/// Enumeration written as `{}`. Values outside the named ones are kept as is.", storage).unwrap();
        writeln!(out, "#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]\npub struct {}(pub {});\n", name, storage).unwrap();
        writeln!(out, "impl {} {{", name).unwrap();
        let mut seen = BTreeSet::new();
        let mut by_value: BTreeMap<i64, String> = BTreeMap::new();
        for (enumerator, value) in &enumerators {
            let constant = const_name(enumerator);
            check_ident(&constant, context)?;
            if !seen.insert(constant.clone()) {
                return Err(format!("{}: two enumerators are named {}", context, constant));
            }
            let range = if p.unsigned {
                0..=((u64::MAX >> (64 - p.bits)).min(i64::MAX as u64) as i64)
            } else {
                (i64::MIN >> (64 - p.bits))..=(i64::MAX >> (64 - p.bits))
            };
            if !range.contains(value) {
                return Err(format!("{}: {} = {} does not fit in {}", context, enumerator, value, storage));
            }
            writeln!(out, "    pub const {}: Self = Self(({}) as {});", constant, value, storage).unwrap();
            by_value.entry(*value).or_insert(constant);
        }
        writeln!(out, "\n    /// Name of the value, if it is a defined one.").unwrap();
        writeln!(out, "    pub fn name(self) -> Option<&'static str> {{\n        match self.0 as i64 {{").unwrap();
        for (value, constant) in &by_value {
            writeln!(out, "            {} => Some(\"{}\"),", value, constant).unwrap();
        }
        writeln!(out, "            _ => None,\n        }}\n    }}\n}}\n").unwrap();
        writeln!(out, "impl std::fmt::Debug for {} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{", name).unwrap();
        writeln!(out, "        match self.name() {{\n            Some(name) => write!(f, \"{}::{{name}}\"),", name).unwrap();
        writeln!(out, "            None => write!(f, \"{}({{}})\", self.0),\n        }}\n    }}\n}}\n", name).unwrap();
        writeln!(out, "impl Wire for {} {{\n    fn write(&self, w: &mut Writer) {{\n        self.0.write(w);\n    }}\n", name).unwrap();
        writeln!(out, "    fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {{\n        Ok(Self(Wire::read(r)?))\n    }}\n}}\n").unwrap();
        if let Some(feature) = self.sample_feature() {
            writeln!(out, "#[cfg(feature = \"{}\")]\nimpl Sample for {} {{\n    fn sample(g: &mut Gen) -> Self {{", feature, name).unwrap();
            if enumerators.is_empty() {
                writeln!(out, "        Self(Sample::sample(g))").unwrap();
            } else {
                let all: Vec<String> = enumerators.iter().map(|(e, _)| format!("Self::{}", const_name(e))).collect();
                writeln!(out, "        [{}][g.below({})]", all.join(", "), all.len()).unwrap();
            }
            writeln!(out, "    }}\n}}\n").unwrap();
        }
        Ok(())
    }
}
