//! What every part of the generator needs: the options, and the types of all
//! the packets with a way to find them.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

use ::flat_ast::{ComplexType, ComplexTypeContent, Element, Occurs, Packet, PacketContent, SimpleType};
use ::heck::ToUpperCamelCase;

use super::names::parse_int;
use super::prim::prim;
use super::RustArgs;

/// A type the XML defines.
#[derive(Clone, Copy)]
pub(super) enum Def<'a> {
    Simple(&'a SimpleType),
    Complex(&'a ComplexType),
}

pub(super) struct Ctx<'a> {
    pub args: &'a RustArgs,
    /// Every named type of every packet, by name. A name always means the
    /// same type: the type registry refuses conflicting definitions.
    pub types: BTreeMap<String, Def<'a>>,
    /// The types some packet uses, directly or through another type. Only
    /// these are generated, so a definition the Rust backend cannot express
    /// stops the build only when something needs it.
    pub needed: BTreeSet<String>,
    /// External enums (`enum="..."` elements): C++ name to Rust path.
    pub externs: BTreeMap<String, String>,
}

impl<'a> Ctx<'a> {
    pub fn new(args: &'a RustArgs, packets: &'a [Packet]) -> Result<Self, String> {
        let mut types = BTreeMap::new();
        for packet in packets {
            for content in packet.contents() {
                match content {
                    PacketContent::Simple(s) => {
                        types.entry(s.name().clone()).or_insert(Def::Simple(s));
                    }
                    PacketContent::Complex(c) => {
                        types.entry(c.name().clone()).or_insert(Def::Complex(c));
                    }
                    _ => {}
                }
            }
        }
        let mut externs = BTreeMap::new();
        for mapping in &args.extern_type {
            let (cpp, rust) = mapping.split_once('=').ok_or_else(|| {
                format!("--extern-type {:?}: expected <C++ type>=<Rust path>", mapping)
            })?;
            externs.insert(cpp.trim().to_owned(), rust.trim().to_owned());
        }
        let mut ctx = Ctx {
            args,
            types,
            needed: BTreeSet::new(),
            externs,
        };
        for packet in packets {
            for content in packet.contents() {
                if let PacketContent::Element(e) = content {
                    ctx.need(e);
                }
            }
        }
        Ok(ctx)
    }

    /// The type an element refers to, if it is one the XML defines.
    pub fn lookup(&self, name: &str) -> Option<(&str, Def<'a>)> {
        let name = name.trim();
        let found = self
            .types
            .get_key_value(name)
            .or_else(|| self.types.get_key_value(&name.to_upper_camel_case()))?;
        Some((found.0.as_str(), *found.1))
    }

    fn need(&mut self, element: &Element) {
        if prim(element.type_()).is_some() {
            return;
        }
        let Some((name, def)) = self.lookup(element.type_()) else {
            // Reported when the element is generated.
            return;
        };
        if !self.needed.insert(name.to_owned()) {
            return;
        }
        if let Def::Complex(complex) = def {
            match complex.content() {
                ComplexTypeContent::Seq(s) => {
                    for e in s.elements() {
                        self.need(e);
                    }
                }
                ComplexTypeContent::Choice(c) => {
                    for e in c.elements() {
                        match c.inline_seqs().get(e.name()) {
                            Some(seq) => {
                                for e in seq.elements() {
                                    self.need(e);
                                }
                            }
                            None => self.need(e),
                        }
                    }
                }
                ComplexTypeContent::Selected(s) => {
                    for case in s.cases() {
                        for e in case.elements() {
                            self.need(e);
                        }
                    }
                }
                ComplexTypeContent::Empty => {}
            }
        }
    }

    /// Rust path of the type of an element, ignoring `occurs`.
    pub fn type_path(&self, element: &Element, context: &str) -> Result<String, String> {
        if let Some(p) = prim(element.type_()) {
            return Ok(p.rust.to_owned());
        }
        if let Some((name, _)) = self.lookup(element.type_()) {
            return Ok(name.to_owned());
        }
        // `enum="..."` on a type the XML does not define: an enum of a C++
        // header, written as that integer.
        if let Some(base) = element.enum_type() {
            let path = self.externs.get(element.type_().trim()).ok_or_else(|| {
                format!(
                    "{}: {} is an external enum; map it with --extern-type {}=<Rust path>",
                    context,
                    element.type_(),
                    element.type_()
                )
            })?;
            if prim(base).filter(|p| p.integer).is_none() {
                return Err(format!("{}: enum base {:?} is not an integer", context, base));
            }
            return Ok(path.clone());
        }
        Err(format!(
            "{}: element {} has unknown type {:?}",
            context,
            element.name(),
            element.type_()
        ))
    }

    /// The number of elements of `occurs="..."`: a number, a constant from
    /// the consts module, or an `Enum::Member` of an enumeration the XML
    /// defines. Returns Rust source for a `usize` constant expression.
    pub fn count(&self, occurs: &Occurs, context: &str) -> Result<String, String> {
        let count = match occurs {
            Occurs::Num(count) => count.trim(),
            Occurs::Unbounded => return Err(format!("{}: not a fixed count", context)),
        };
        if let Some(number) = parse_int(count) {
            return Ok(number.to_string());
        }
        if let Some((ty, member)) = count.split_once("::") {
            if let Some((_, Def::Simple(simple))) = self.lookup(ty) {
                if let Some(value) = enumerators(simple).into_iter().find(|(n, _)| n == member) {
                    return u64::try_from(value.1)
                        .map(|v| v.to_string())
                        .map_err(|_| format!("{}: element count {} is negative", context, count));
                }
            }
            return Err(format!("{}: cannot resolve the element count {:?}", context, count));
        }
        if count.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Ok(format!("{}::{} as usize", self.args.consts_path, count));
        }
        Err(format!("{}: cannot resolve the element count {:?}", context, count))
    }
}

/// The enumerators of a simple type with their values.
pub(super) fn enumerators(simple: &SimpleType) -> Vec<(String, i64)> {
    use ::flat_ast::{RestrictionContent, SimpleTypeContent};
    let mut out = Vec::new();
    for content in simple.contents() {
        let SimpleTypeContent::Restriction(r) = content;
        for item in r.contents() {
            if let RestrictionContent::Enumeration(e) = item {
                out.push((e.value().clone(), e.id()));
            }
        }
    }
    out
}

/// Base type and `<length>` of a simple type.
pub(super) fn restriction(simple: &SimpleType) -> (String, Option<u32>) {
    use ::flat_ast::{RestrictionContent, SimpleTypeContent};
    let mut base = String::new();
    let mut length = None;
    if let Some(SimpleTypeContent::Restriction(r)) = simple.contents().first() {
        base = r.base().trim().to_owned();
        for item in r.contents() {
            if let RestrictionContent::Length(l) = item {
                length = Some(*l);
            }
        }
    }
    (base, length)
}
