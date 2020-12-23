use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;

pub (crate) struct CodeHeaderGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>,
    version: String
}

impl<'a, W: Write> CodeHeaderGenerator<'a, W> {
    pub fn new(writer: &'a mut ::writer::Writer<W>, version: String) -> Self {
        Self {
            writer,
            version
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
        let version = self.version.clone();
        cg!(self, "/* Generated with IDL v{} */\n", version);
        self.doc(packet.doc())?;
        cg!(self);
        cg!(self, r#"#include "packetfactory.h""#);
        cg!(self);
        cg!(self, "#ifndef JSON_USE_IMPLICIT_CONVERSIONS");
        cg!(self, "#define JSON_USE_IMPLICIT_CONVERSIONS 0");
        cg!(self, "#include \"json.hpp\"");
        cg!(self, "#endif");
        
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
"#);
        
        cg!(self, "class {} : public CRosePacket {{", packet.class_name());
        self.indent();
        cg!(self, "public:");
        self.indent();
        cg!(self, "static constexpr ePacketType PACKET_ID = ePacketType::{};", packet.type_());
        cg!(self, "{}();", packet.class_name());
        cg!(self, "{}(CRoseReader reader);", packet.class_name());
        cg!(self, "{0}({0}&&) = default;", packet.class_name());
        cg!(self, "{0}& operator=({0}&&) = default;", packet.class_name());
        cg!(self, "~{}() = default;", packet.class_name());
        cg!(self);
        cg!(self, "static constexpr size_t size();");
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
                    self.elem_setter(elem, packet.class_name())?;
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
        cg!(self, "virtual bool pack(CRoseBasePolicy&) const override;");
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

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(s) => {
                    self.simple_type_to_json(s)?;
                    self.simple_type_from_json(s)?;
                },
                _ => {}
            }
        }

        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(c) => {
                    self.complex_type_to_json(c)?;
                    self.complex_type_from_json(c)?;
                },
                _ => {}
            }
        }

        self.packet_to_json(packet)?;
        cg!(self);
        self.packet_from_json(packet)?;
        cg!(self, "}}\n}}");
        Ok(())
    }

    fn packet_to_json(&mut self, packet: &Packet) -> Result<()> {
        cg!(self, "void to_json(nlohmann::json& j, const {}& data);", packet.class_name());
        Ok(())
    }

    fn packet_from_json(&mut self, _packet: &Packet) -> Result<()> {
        Ok(())
    }

    fn simple_type_to_json(&mut self, element: &SimpleType) -> Result<()> {
        cg!(self, "void to_json(nlohmann::json& j, const {}& data);", element.name());
        Ok(())
    }

    fn simple_type_from_json(&mut self, _element: &SimpleType) -> Result<()> {
        Ok(())
    }

    fn complex_type_to_json(&mut self, element: &ComplexType) -> Result<()> {
        if element.inline() == true {
            return Ok(());
        }
        cg!(self, "void to_json(nlohmann::json& j, const {}& data);", element.name());
        Ok(())
    }

    fn complex_type_from_json(&mut self, _element: &ComplexType) -> Result<()> {
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
        use ::flat_ast::ComplexTypeContent::*;
        if complex.inline() == false {
            cg!(self, "struct {} : public ISerialize {{", complex.name());
            self.indent();
            cg!(self, "virtual bool read(CRoseReader&) override;");
            cg!(self, "virtual bool write(CRoseBasePolicy&) const override;");
            cg!(self);
            cg!(self, "static constexpr size_t size();");
            cg!(self);
            match complex.content() {
                Seq(ref s) => {
                    for elem in s.elements() {
                        self.elem_setter(elem, complex.name())?;
                        self.elem_getter(elem)?;
                    }
                    cg!(self);
                    cg!(self, "private:");
                    self.indent();
                    for elem in s.elements() {
                        self.element(elem)?;
                    }
                    self.dedent();
                },
                Choice(ref c) => {
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            for e in seq.elements() {
                                self.elem_setter(e, complex.name())?;
                                self.elem_getter(e)?;
                            }
                        } else {
                            self.elem_setter(elem, complex.name())?;
                            self.elem_getter(elem)?;
                        }
                    }
                    cg!(self);
                    cg!(self, "private:");
                    self.indent();
                    cg!(self, "union {{");
                    self.indent();
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            cg!(self, "PACK(struct {{");
                            self.indent();
                            for e in seq.elements() {
                                self.element(e)?;
                            }
                            self.dedent();
                            cg!(self, "}});");
                        } else {
                            self.element(elem)?;
                        }
                    }
                    self.dedent();
                    cg!(self, "}} data;");
                    self.dedent();
                },
                Empty => {}
            }
            self.dedent();
            cg!(self, "}};");
            cg!(self);
        }
        Ok(())
    }

    fn simple_type(&mut self, simple: &SimpleType) -> Result<()> {
        cg!(self);
        self.doc(simple.doc())?;
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name())?
            }
        }
        Ok(())
    }

    fn element(&mut self, elem: &Element) -> Result<()> {
        self.doc(elem.doc())?;

        if let Some(bitset) = elem.bitset() {
            if bitset.start == 0 {
                cg!(self, "std::bitset<{}> {};", bitset.size, bitset.name);
            }
            return Ok(());
        }

        let (type_, bits) = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            let type_ = match o {
                Unbounded => format!("std::vector<{}>", elem.type_()),
                Num(n) => format!("std::array<{}, {}>", elem.type_(), n)
            };
            (type_, "".to_string())
        } else {
            let bits = elem.bits().map_or_else(|| "".to_string(), |b| format!(" : {}", b));
            (elem.type_().to_owned(), bits)
        };
        let default = match elem.init() {
            self::ElementInitValue::Default(d) => " = ".to_string() + d,
            _ => "".to_string()
        };
        cg!(self, "{} {}{}{};", type_, elem.name(), bits, default);
        Ok(())
    }

    fn elem_setter(&mut self, elem: &Element, class: &str) -> Result<()> {
        let reference = if elem.reference() { "&" } else { "" };
        use ::flat_ast::Occurs::*;
        let type_ = if let Some(ref o) = elem.occurs() {
            match o {
                Unbounded => format!("std::vector<{}>", elem.type_()),
                Num(n) => {
                    format!("std::array<{}, {}>", elem.type_(), n)
                }
            }
        } else {
            elem.type_().to_owned()
        };
        cg!(self, "{}& set_{}(const {}{});", class, elem.name(), type_, reference);
        if let Some(ref o) = elem.occurs() {
            match o {
                Unbounded => { cg!(self, "{}& add_{}(const {}{});", class, elem.name(), elem.type_(), reference); },
                Num(_) => { cg!(self, "{}& set_{}(const {}{}, size_t index);", class, elem.name(), elem.type_(), reference); }
            }
        }
        Ok(())
    }

    fn elem_getter(&mut self, elem: &Element) -> Result<()> {
        let reference = if elem.reference() { "&" } else { "" };
        let type_ = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            match o {
                Unbounded => format!("std::vector<{}>", elem.type_()),
                Num(n) => {
                    format!("std::array<{}, {}>", elem.type_(), n)
                }
            }
        } else {
            elem.type_().to_owned()
        };
        let is_const = if elem.reference() { "const " } else { "" };
        cg!(self, "{}{}{} get_{}() const;", is_const, type_, reference, elem.name());
        let is_const = if elem.reference() { "const " } else { "" };
        if elem.occurs().is_some() {
            cg!(self, "{}{}{} get_{}(size_t index) const;", is_const, elem.type_(), reference, elem.name());
        }
        Ok(())
    }

    fn create(&mut self, packet: &Packet) -> Result<()> {
        let args = packet.contents().iter().map(|elem| {
            use self::PacketContent::*;
            match elem {
                Element(ref e) => match e.init() {
                        self::ElementInitValue::Create => {
                            if let Some(ref o) = e.occurs() {
                                use ::flat_ast::Occurs::*;
                                let t = match o {
                                    Unbounded => format!("std::vector<{}>", e.type_()),
                                    Num(n) => format!("std::array<{}, {}>", e.type_(), n)
                                };
                                "const ".to_owned() + &t + &format!("& {}, ", e.name())
                            } else {
                                "const ".to_owned() + e.type_() + &format!("& {}, ", e.name())
                            }
                        },
                        _ => "".to_owned()
                    },
                _ => "".to_string()
            }
        }).collect::<String>();
        let args = if args.len() != 0 {
            &args[..args.len() - 2]
        } else {
            &args
        };
        cg!(self, "static {} create({});", packet.class_name(), args);
        cg!(self, "static {} create(const uint8_t* buffer);", packet.class_name());
        cg!(self, "static std::unique_ptr<{}> allocate(const uint8_t* buffer);", packet.class_name());
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
            cg!(self, "{}();", name);
            cg!(self, "{}({});", name, base);
            cg!(self, "{0}(const {0}&) = default;", name);
            cg!(self, "{0}({0}&&) = default;", name);
            cg!(self, "{0}& operator=(const {0}&) = default;", name);
            cg!(self, "{0}& operator=({0}&&) = default;", name);
            cg!(self, "virtual ~{}() = default;", name);
            cg!(self);
            cg!(self, "static constexpr size_t size();");
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
