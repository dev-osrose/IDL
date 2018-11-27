use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! cg {
        ($v:expr) => (($v).write("")?);
        ($v:expr, $fmt:expr) => (($v).write(format!($fmt))?);
        ($v:expr, $fmt:expr, $($arg:tt)*) => (($v).write(format!($fmt, $($arg)*))?);
    }
}

pub (super) struct CodeHeaderGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>
}

impl<'a, W: Write> CodeHeaderGenerator<'a, W> {
    pub fn new(writer: &'a mut ::writer::Writer<W>) -> Self {
        Self {
            writer
        }
    }

    fn indent(&mut self) {
        self.writer.indent();
    }

    fn dedent(&mut self) {
        self.writer.dedent();
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.writer.write(val)?;
        Ok(self)
    }

    pub fn generate(&mut self, packet: &Packet) -> Result<()> {
        cg!(self, "#pragma once\n");
        self.doc(packet.doc())?;
        cg!(self);
        cg!(self, r#"#include "packetfactory.h""#);
        
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Include(ref inc, system) => { 
                    if *system {
                        cg!(self, r#"#include <{}>"#, inc);
                    } else {
                        cg!(self, r#"#include "{}""#, inc);
                    }
                },
                _ => {}
            };
        }

        cg!(self);
        cg!(self, r#"namespace RoseCommon {{
namespace Packet {{

REGISTER_RECV_PACKET(ePacketType::{0}, {1})
REGISTER_SEND_PACKET(ePacketType::{0}, {1})"#,
            packet.type_(), packet.class_name());
        
        cg!(self, "class {} : public CRosePacket {{", packet.class_name());
        self.indent();
        cg!(self, "public:");
        self.indent();
        cg!(self, "{}();", packet.class_name());
        cg!(self, "{}(CRoseReader reader);", packet.class_name());
        cg!(self, "{0}({0}&&) = default;", packet.class_name());
        cg!(self, "{0}& operator=({0}&&) = default;", packet.class_name());
        cg!(self, "~{}() = default;", packet.class_name());
        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(ref simple) => self.simple_type(simple)?,
                _ => {}
            };
        }

        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(ref complex) => self.complex_type(complex)?,
                _ => {}
            };
        }

        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => {
                    self.elem_setter(elem)?;
                    self.elem_getter(elem)?;
                },
                _ => {}
            };
        }

        cg!(self);
        cg!(self);
        self.create(packet)?;

        self.dedent();
        cg!(self);
        cg!(self, "protected:");
        self.indent();
        cg!(self, "virtual void pack(CRoseBasePolicy&) const override;");
        self.dedent();
        cg!(self);
        cg!(self, "private:");
        self.indent();

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => self.element(elem)?,
                _ => {}
            };
        }

        self.dedent();
        self.dedent();
        cg!(self, "}};");
        cg!(self);
        cg!(self, "}}\n}}");
        Ok(())
    }

    fn doc(&mut self, doc: &Option<String>) -> Result<()> {
        match doc {
            None => (),
            Some(doc) => {
                for line in doc.lines() {
                    match line.trim() {
                        "" => (),
                        line => {
                            cg!(self, "// {}", line);
                        }
                    }
                }
            }
        };
        Ok(())
    }

    fn complex_type(&mut self, complex: &ComplexType) -> Result<()> {
        Ok(())
    }

    fn simple_type(&mut self, simple: &SimpleType) -> Result<()> {
        cg!(self);
        self.doc(simple.doc())?;
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name())?            }
        }
        Ok(())
    }

    fn element(&mut self, elem: &Element) -> Result<()> {
        self.doc(elem.doc())?;

        cg!(self, "{} {};", elem.type_(), elem.name());
        Ok(())
    }

    fn elem_setter(&mut self, elem: &Element) -> Result<()> {
        let reference = if elem.reference() { "&" } else { "" };
        cg!(self, "void set_{}({}{});", elem.name(), elem.type_(), reference);
        Ok(())
    }

    fn elem_getter(&mut self, elem: &Element) -> Result<()> {
        let reference = if elem.reference() { "&" } else { "" };
        cg!(self, "{}{} get_{}() const;", elem.type_(), reference, elem.name());
        Ok(())
    }

    fn create(&mut self, packet: &Packet) -> Result<()> {
        let args = packet.contents().iter().map(|elem| {
            use self::PacketContent::*;
            match elem {
                Element(ref e) => e.type_().clone() + ", ",
                _ => "".to_string()
            }
        }).collect::<String>();
        let args = &args[..args.len() - 2];
        cg!(self, "static {} create({});", packet.class_name(), args);
        Ok(())
    }

    fn restrict(&mut self, restrict: &Restriction, name: &str) -> Result<()> {
        use self::RestrictionContent::*;
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();
        self.doc(restrict.doc())?;
        let base = restrict.base();

        if is_enum {
            cg!(self, "enum {} : {} {{", name, base);
            self.indent();
            for content in restrict.contents() {
                if let Enumeration(en) = content {
                    self.doc(en.doc())?;
                    cg!(self, "{} = {},", en.value(), en.id());
                }
            }
        } else {
            cg!(self, "struct {} : public ISerialize {{", name);
            self.indent();
            cg!(self, "explicit {}();", name);
            cg!(self, "explicit {}({});", name, base);
            cg!(self, "{0}(const {0}&) = default;", name);
            cg!(self, "{0}({0}&&) = default;", name);
            cg!(self, "virtual ~{}() = default;", name);
            cg!(self);
            cg!(self, "operator {}() const {{ return {}; }}", base, name.to_string().to_snake_case());
            cg!(self, "bool isValid() const {{ return is_valid; }}");
            cg!(self);
            cg!(self, "virtual bool read(CRoseReader&) override;");
            cg!(self, "virtual bool write(CRoseBasePolicy&) const override;");
            cg!(self);
            cg!(self, "private:");
            self.indent();
            cg!(self, "{} {};", base, name.to_string().to_snake_case());
            cg!(self, "bool is_valid;");
            self.dedent();
        }

        self.dedent();
        cg!(self, "}};");
        Ok(())
    }
}

pub (super) struct CodeSourceGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>
}

impl<'a, W: Write> CodeSourceGenerator<'a, W> {
    pub fn new(writer: &'a mut ::writer::Writer<W>) -> Self {
        Self {
            writer
        }
    }

    fn indent(&mut self) {
        self.writer.indent();
    }

    fn dedent(&mut self) {
        self.writer.dedent();
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        self.writer.write(val)?;
        Ok(self)
    }

    pub fn generate(&mut self, packet: &Packet) -> Result<()> {
        cg!(self, r#"#include "{}.h""#, packet.filename());
        cg!(self);
        cg!(self, "using namespace RoseCommon::Packet;");
        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(simple) => self.simple_type(simple, packet.class_name())?,
                _ => {}
            }
        }

        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(complex) => self.complex_type(complex)?,
                _ => {}
            }
        }

        cg!(self);
        cg!(self, "{0}::{0}() : CRosePacket(ePacketType::{1}) {{}}", packet.class_name(), packet.type_());
        cg!(self);
        cg!(self, "{0}::{0}(CRoseReader reader) : CRosePacket(reader) {{", packet.class_name());
        self.indent();
        let iserialize = packet.contents().iter().filter_map(|elem| if PacketContent::is_type(elem) { PacketContent::type_from_name(elem) } else { None }).collect::<std::collections::HashSet<String>>();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(elem) => {
                    let base = if let Some(ref enum_type) = elem.enum_type() {
                        enum_type.clone()
                    } else if iserialize.contains(&elem.type_().to_owned().to_camel_case()) {
                        "iserialize".to_owned()
                    } else {
                        clean_base(elem.type_())
                    };
                    self.write_if_else(&format!("!reader.get_{}({})", base, elem.name()), &[
                        "return;"
                    ], None)?;
                },
                _ => {}
            }
        }
        self.dedent();
        cg!(self, "}}");
        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(elem) => {
                    self.elem_setter(elem)?;
                    self.elem_getter(elem)?;
                },
                _ => {}
            }
        }

        self.create(packet)?;
        cg!(self);
        self.pack(packet)?;
        cg!(self);
        Ok(())
    }

    fn complex_type(&mut self, complex: &ComplexType) -> Result<()> {
        Ok(())
    }

    fn simple_type(&mut self, simple: &SimpleType, class_name: &str) -> Result<()> {
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name(), class_name)?            }
        }
        Ok(())
    }

    fn element(&mut self, elem: &Element) -> Result<()> {
        Ok(())
    }

    fn elem_setter(&mut self, elem: &Element) -> Result<()> {
        Ok(())
    }

    fn elem_getter(&mut self, elem: &Element) -> Result<()> {
        Ok(())
    }

    fn create(&mut self, packet: &Packet) -> Result<()> {
        let args = packet.contents().iter().map(|elem| {
            use self::PacketContent::*;
            match elem {
                Element(ref e) => e.type_().clone() + ", ",
                _ => "".to_string()
            }
        }).collect::<String>();
        let args = &args[..args.len() - 2];
        cg!(self, "{0} {0}::create({1}) {{", packet.class_name(), args);
        self.indent();
        // TODO: finish create
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn pack(&mut self, packet: &Packet) -> Result<()> {
        cg!(self, "void {}::pack(CRoseBasePolicy& writer) const {{", packet.class_name());
        self.indent();
        let iserialize = packet.contents().iter().filter_map(PacketContent::type_from_name).collect::<std::collections::HashSet<String>>();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(elem) => {
                    let base = if iserialize.contains(&elem.type_().to_owned().to_camel_case()) {
                        "iserialize".to_owned()
                    } else {
                        clean_base(elem.type_())
                    };
                    self.write_if_else(&format!("!writer.set_{}({})", base, elem.name()), &[
                        "return;"
                    ], None)?;
                },
                _ => {}
            }
        }
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn restrict(&mut self, restrict: &Restriction, name: &str, class_name: &str) -> Result<()> {
        use self::RestrictionContent::*;
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();
        let base = restrict.base();

        if !is_enum {
            let data = name.to_string().to_snake_case();
            cg!(self, "{0}::{1}::{1}() : is_valid(false) {{}}", class_name, name);
            cg!(self);
            cg!(self, "{0}::{1}::{1}({2} data) : {3}(data), is_valid(false) {{", class_name, name, base, data);
            self.indent();
            let mut wrote = false;
            for content in restrict.contents() {
                if !wrote {
                    cg!(self, "bool valid = false;");
                }
                wrote = true;
                match content {
                    Length(l) => {
                        self.write_if_else(&format!("{}.size() > {}", data, l), &[
                            &format!("{}.resize({});", data, l),
                            "valid &= true;"
                        ], Some(&[
                            "valid &= true;"
                        ]))?;
                    },
                    MinValue(v) => {
                        cg!(self, "valid &= {} > {};", data, v);
                    },
                    MaxValue(v) => {
                        cg!(self, "valid &= {} < {};", data,  v);
                    },
                    _ => panic!("enumeration in restrict when there shouldn't be one")
                }
            }
            if !wrote {
                cg!(self, "is_valid = true;");
            } else {
                cg!(self, "is_valid = valid;");
            }
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            cg!(self, "bool {}::{}::read(CRoseReader& reader) {{", class_name, name);
            self.indent();
            let base = clean_base(base);
            let mut wrote = false;
            for content in restrict.contents() {
                if !wrote {
                    cg!(self, "bool valid = false;");
                }
                wrote = true;
                match content {
                    Length(l) => {
                        self.write_if_else(&format!("!reader.get_{}({}, {})", base, data, l),&[
                            "return false;"
                        ], Some(&[
                            "valid &= true;"
                        ]))?;
                    },
                    MinValue(v) => {
                        self.write_if_else(&format!("!reader.get_{}({})", base, data), &[
                            "return false;"
                        ], Some(&[
                            &format!("valid &= {} > {};", data, v)
                        ]))?;
                    },
                    MaxValue(v) => {
                        self.write_if_else(&format!("!reader.get_{}({})", base, data), &[
                            "return false;"
                        ], Some(&[
                            &format!("valid &= {} < {};", data, v)
                        ]))?;
                    },
                    _ => panic!("enumeration in restrict when there shouldn't be one")
                }
            }
            if !wrote {
                cg!(self, "if (!reader.get_{}({})) return false;", base, data);
                cg!(self, "is_valid = true;");
            } else {
                cg!(self, "is_valid = valid;");
            }
            cg!(self, "return true;");
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            cg!(self, "bool {}::{}::write(CRoseBasePolicy& writer) const {{", class_name, name);
            self.indent();
            for content in restrict.contents() {
                match content {
                    Length(l) => {
                        self.write_if_else(&format!("!writer.set_{}({}, {})", base, data, l),&[
                            "return false;"
                        ], None)?;
                    },
                    MinValue(_) => {
                        self.write_if_else(&format!("!writer.set_{}({})", base, data), &[
                            "return false;"
                        ], None)?;
                    },
                    MaxValue(_) => {
                        self.write_if_else(&format!("!writer.set_{}({})", base, data), &[
                            "return false;"
                        ], None)?;
                    },
                    _ => panic!("enumeration in restrict when there shouldn't be one")
                }
            }
            cg!(self, "return true;");
            self.dedent();
            cg!(self, "}}");
        }
        Ok(())
    }

    fn write_if_else(&mut self, condition: &str, if_branch: &[&str], else_branch: Option<&[&str]>) -> Result<()> {
        cg!(self, "if ({}) {{", condition);
        self.indent();
        for branch in if_branch {
            cg!(self, "{}", branch);
        }
        self.dedent();
        if let Some(else_branch) = else_branch {
            cg!(self, "}} else {{");
            self.indent();
            for branch in else_branch {
                cg!(self, "{}", branch);
            }
            self.dedent();
        }
        cg!(self, "}}");
        Ok(())
    }
}

fn clean_base(base: &str) -> String {
    if base.contains("::") {
        base.split("::").skip(1).collect()
    } else {
        base.to_string()
    }
}