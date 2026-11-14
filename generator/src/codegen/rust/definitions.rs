//! `<definitions>`: a module of their own, beside the packets, with the
//! enumerations (as in `shared_types.rs`) and the constants.

use std::fmt::Write as _;

use ::codegen::{const_value, ConstValue};
use ::flat_ast::{Constant, Definitions, DefinitionsContent};

use super::context::Ctx;
use super::names::check_ident;
use super::prim::prim;
use super::{doc_comment, files, Result};

impl<'a> Ctx<'a> {
    pub(super) fn definitions(&self, version: &str, definitions: &Definitions) -> Result<String> {
        let context = definitions.name();
        check_ident(context, context)?;
        let mut out = files::module_header(version, context, definitions.doc(), self.args, false);
        for content in definitions.contents() {
            match content {
                DefinitionsContent::Simple(simple) => self.simple_type(&mut out, simple, context)?,
                DefinitionsContent::Constant(constant) => constant_item(&mut out, constant, context)?,
            }
        }
        Ok(out)
    }
}

fn constant_item(out: &mut String, constant: &Constant, context: &str) -> Result<()> {
    let name = constant.name();
    check_ident(name, context)?;
    let value = match const_value(constant).map_err(|e| format!("{}: {}", context, e))? {
        ConstValue::Integer(_) => constant.value().trim().to_owned(),
        ConstValue::Float(v) => format!("{:?}", v),
    };
    // const_value accepted the type, so it is a number type.
    let ty = prim(constant.type_()).map(|p| p.rust).unwrap_or_default();
    doc_comment(out, "", constant.doc());
    writeln!(out, "pub const {}: {} = {};", name, ty, value).unwrap();
    Ok(())
}
