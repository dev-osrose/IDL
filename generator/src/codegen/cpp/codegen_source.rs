use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;
use std::collections::HashSet;

pub (crate) struct CodeSourceGenerator<'a, W: Write + 'a> {
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
        cg!(self, "using namespace RoseCommon;");
        cg!(self, "using namespace RoseCommon::Packet;");
        cg!(self);

        let iserialize = packet.contents().iter().filter_map(|elem| {
            if PacketContent::is_type(elem) {
                PacketContent::type_from_name(elem)
            } else {
                match elem {
                    PacketContent::Element(ref e) => {
                        match e.type_().as_ref() {
                            "int8_t"
                            | "uint8_t"
                            | "int16_t"
                            | "uint16_t"
                            | "int32_t"
                            | "uint32_t"
                            | "int64_t"
                            | "uint64_t"
                            | "char"
                            | "float"
                            | "double"
                            | "std::string" => None,
                            _ => Some(e.type_().to_string())
                        }
                    },
                    _ => None
                }
            }
        }).collect::<::std::collections::HashSet<String>>();

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
                Complex(complex) => self.complex_type(complex, packet.class_name(), &iserialize)?,
                _ => {}
            }
        }

        cg!(self);
        cg!(self, "{0}::{0}() : CRosePacket({0}::PACKET_ID) {{", packet.class_name());
        if packet.class_name().starts_with("Srv") {
            self.indent();
            cg!(self, "set_server_packet();");
            self.dedent();
            cg!(self, "}}");
        } else {
            cg!(self, "}}");
        }
        cg!(self);
        cg!(self, "{0}::{0}(CRoseReader reader) : CRosePacket(reader) {{", packet.class_name());
        self.indent();
        if packet.class_name().starts_with("Srv") {
            cg!(self, "set_server_packet();");
            cg!(self);
        }
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(elem) => {
                    let base = if let Some(ref read_write) = elem.read_write() {
                        read_write.clone()
                    } else if let Some(ref enum_type) = elem.enum_type() {
                        enum_type.clone()
                    } else if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) {
                        "iserialize".to_owned()
                    } else {
                        clean_base(elem.type_())
                    };

                    let name = elem.name().to_owned();

                    if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded => {
                                cg!(self, "{{");
                                self.indent();
                                let class_base = if elem.is_defined() {
                                    packet.class_name().to_owned() + "::"
                                } else {
                                    "".to_owned()
                                };
                                let enum_name = if let Some(ref enum_type) = elem.enum_type() {
                                    format!("({}&)", enum_type)
                                } else {
                                    "".to_owned()
                                };
                                if let Some(ref s) = elem.size_occurs() {
                                    cg!(self, "{} size;", s);
                                    self.write_if_else(&format!("!reader.get_{}(size)", s), &[
                                        "return;"
                                    ], None)?;
                                    cg!(self, "while (size-- > 0) {{");
                                    self.indent();
                                    cg!(self, "{} elem;", class_base + elem.type_());
                                    self.write_if_else(&format!("!reader.get_{}({}elem)", base, enum_name), &[
                                        "return;"
                                    ], None)?;
                                    cg!(self, "{}.push_back(elem);", elem.name());
                                    self.dedent();
                                    cg!(self, "}}");
                                } else {
                                    cg!(self, "{} elem;", class_base + elem.type_());
                                    cg!(self, "while (reader.get_{}({}elem)) {{", base, enum_name);
                                    self.indent();
                                    cg!(self, "{}.push_back(elem);", elem.name());
                                    self.dedent();
                                    cg!(self, "}}");
                                }
                                self.dedent();
                                cg!(self, "}}");
                            },
                            Num(n) => {
                                cg!(self, "for (size_t index = 0; index < {}; ++index) {{", n);
                                self.indent();
                                self.write_if_else(&format!("!reader.get_{}({}[index])", base, name), &[
                                        "return;"
                                    ], None)?;
                                self.dedent();
                                cg!(self, "}}");
                            }
                        }
                    } else if let Some(ref enum_type) = elem.enum_type() {
                        // Introduce a temprorary variable to avoid reference aliasing issues
                        let temp_name = format!("{}_temp", elem.name());
                        cg!(self, "{} {};", enum_type, temp_name);
                        self.write_if_else(&format!("!reader.get_{}({})", enum_type, temp_name), &[
                            "return;"
                        ], None)?;
                        cg!(self, "{} = static_cast<{}>({});", elem.name(), elem.type_(), temp_name);
                        cg!(self);
                    } else {
                        let name = if let Some(bitset) = elem.bitset() {
                            if bitset.start == 0 {
                                Some(&bitset.name)
                            } else {
                                None
                            }
                        } else {
                            Some(elem.name())
                        };
                        let base = if elem.bitset().is_some() {
                            "bitset".to_owned()
                        } else {
                            base
                        };
                        if let Some(name) = name {
                            self.write_if_else(&format!("!reader.get_{}({})", base, name), &[
                                    "return;"
                                ], None)?;
                        }
                    }
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
                    self.elem_setter(elem, packet.class_name(), false)?;
                    self.elem_getter(elem, packet.class_name(), false)?;
                },
                _ => {}
            }
        }

        self.create(packet)?;
        cg!(self);
        self.pack(packet, &iserialize)?;
        cg!(self);
        cg!(self, "constexpr size_t {}::size() {{", packet.class_name());
        self.indent();
        let iserialize = packet.contents().iter().filter_map(|elem| match elem {
            self::PacketContent::Element(elem) => if elem.is_defined() { Some(elem.type_().to_owned()) } else { None },
            _ => None
        }).collect::<::std::collections::HashSet<String>>();
        cg!(self, "size_t size = 0;");
        for elem in packet.contents() {
            match elem {
                self::PacketContent::Element(elem) => {
                    if elem.type_() == "std::string" {
                        continue;
                    }
                    if let Some(ref size) = elem.size_occurs() {
                        cg!(self, "size += sizeof({}); // {}", size, elem.name());
                    }
                    let rhs = if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) && elem.enum_type().is_none() {
                        format!("{}::size()", elem.type_())
                    } else {
                        let rhs = elem.bitset().as_ref().map_or(Some(format!("sizeof({})", elem.type_())), |bitset| if bitset.start == 0 {
                            Some(format!("{} / 8", bitset.size))
                        } else {
                            None
                        });
                        
                        if rhs.is_none() {
                            continue;
                        }
                        rhs.unwrap()
                    };
                    let rhs = if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded => rhs,
                            Num(n) => rhs + " * " + n
                        }
                    } else {
                        rhs
                    };
                    let name = if let Some(bitset) = elem.bitset() {
                        &bitset.name
                    } else {
                        elem.name()
                    };
                    cg!(self, "size += {}; // {}", rhs, name);
                },
                _ => {}
            }
        }
        cg!(self, "return size;");
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(s) => {
                    self.simple_type_to_json(packet.class_name(), s)?;
                    self.simple_type_from_json(packet.class_name(), s)?;
                },
                _ => {}
            }
        }

        cg!(self);

        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(c) => {
                    self.complex_type_to_json(packet.class_name(), c)?;
                    self.complex_type_from_json(packet.class_name(), c)?;
                },
                _ => {}
            }
        }

        self.packet_to_json(packet)?;
        cg!(self);
        self.packet_from_json(packet)?;
        Ok(())
    }

    fn packet_to_json(&mut self, packet: &Packet) -> Result<()> {
        cg!(self, "void RoseCommon::Packet::to_json(nlohmann::json& j, const {}& data) {{", packet.class_name());
        self.indent();
        cg!(self, "j = nlohmann::json{{");
        self.indent();
        cg!(self, "{{ \"metadata\", {{ {{ \"packet\", \"{}\" }}, {{ \"size\", data.get_size() }} }} }},", packet.type_());
        cg!(self, "{{ \"fields\", {{");
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(e) => {
                    let bitfield = if e.bitset().is_some() {
                        " == 1"
                    } else {
                        ""
                    };
                    cg!(self, "{{ \"{0}\", data.get_{0}(){1} }},", e.name(), bitfield);
                }
                _ => {}
            }
        }
        self.dedent();
        cg!(self, "}} }}");
        self.dedent();
        cg!(self, "}};");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn packet_from_json(&mut self, _packet: &Packet) -> Result<()> {
        Ok(())
    }

    fn simple_type_to_json(&mut self, packet_name:&str, element: &SimpleType) -> Result<()> {
        cg!(self, "void RoseCommon::Packet::to_json(nlohmann::json& j, const {}::{}& data) {{", packet_name, element.name());
        self.indent();
        cg!(self, "j = nlohmann::json{{");
        self.indent();
        for content in element.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => {
                    use self::RestrictionContent::*;
                    let is_enum = res.contents().iter().find(|content| match content {
                        Enumeration(_) => true,
                        _ => false
                    }).is_some();
                    if is_enum {
                        cg!(self, "{{ \"value\", static_cast<{}>(data) }},", res.base());
                    } else {
                        cg!(self, "{{ \"value\", data.operator {}() }},", res.base());
                    }
                }
            }
        }
        self.dedent();
        cg!(self, "}};");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn simple_type_from_json(&mut self, _packet_name: &str, _element: &SimpleType) -> Result<()> {
        Ok(())
    }

    fn complex_type_to_json(&mut self, packet_name: &str, element: &ComplexType) -> Result<()> {
        if element.inline() == true {
            return Ok(());
        }
        cg!(self, "void RoseCommon::Packet::to_json(nlohmann::json& j, const {}::{}& data) {{", packet_name, element.name());
        self.indent();
        use ::flat_ast::ComplexTypeContent::*;
        cg!(self, "j = nlohmann::json{{");
        self.indent();
        match element.content() {
            Seq(ref s) => {
                for elem in s.elements() {
                    let bitfield = if elem.bitset().is_some() {
                        " == 1"
                    } else {
                        ""
                    };
                    cg!(self, "{{ \"{0}\", data.get_{0}(){1} }},", elem.name(), bitfield);
                }
            },
            Choice(ref c) => {
                for elem in c.elements() {
                    if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                        for e in seq.elements() {
                            let bitfield = if e.bitset().is_some() {
                                " == 1"
                            } else {
                                ""
                            };
                            cg!(self, "{{ \"{0}\", data.get_{0}(){1} }},", e.name(), bitfield);
                        }
                    } else {
                        let bitfield = if elem.bitset().is_some() {
                            " == 1"
                        } else {
                            ""
                        };
                        cg!(self, "{{ \"{0}\", data.get_{0}(){1} }},", elem.name(), bitfield);
                    }
                }
            },
            Empty => {}
        }
        self.dedent();
        cg!(self, "}};");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn complex_type_from_json(&mut self, _packet_name: &str, _element: &ComplexType) -> Result<()> {
        Ok(())
    }

    fn complex_type(&mut self, complex: &ComplexType, class_name: &str, iserialize: &HashSet<String>) -> Result<()> {
        use ::flat_ast::ComplexTypeContent::*;
        let class_name = class_name.to_owned() + "::" + complex.name();
        if complex.inline() == false {
            match complex.content() {
                Seq(ref s) => {
                    for elem in s.elements() {
                        self.elem_setter(elem, &class_name, false)?;
                        self.elem_getter(elem, &class_name, false)?;
                    }
                    self.pack_sequence(s, &class_name, iserialize)?;
                    cg!(self);
                    self.read_sequence(s, &class_name, iserialize)?;
                    cg!(self);
                    cg!(self, "constexpr size_t {}::size() {{", class_name);
                    self.indent();
                    cg!(self, "size_t size = 0;");
                    for elem in s.elements() {
                        if elem.type_() == "std::string" {
                            continue;
                        }
                        let rhs = if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) {
                            format!("{}::size()", elem.type_())
                        } else {
                            let rhs = elem.bitset().as_ref().map_or(Some(format!("sizeof({})", elem.type_())), |bitset| if bitset.start == 0 {
                                Some(format!("{} / 8", bitset.size))
                            } else {
                                None
                            });
                            
                            if rhs.is_none() {
                                continue;
                            }
                            rhs.unwrap()
                        };
                        let rhs = if let Some(ref o) = elem.occurs() {
                            use ::flat_ast::Occurs::*;
                            match o {
                                Unbounded => rhs,
                                Num(n) => rhs + " * " + n
                            }
                        } else {
                            rhs
                        };
                        let name = if let Some(bitset) = elem.bitset() {
                            &bitset.name
                        } else {
                            elem.name()
                        };
                        cg!(self, "size += {}; // {}", rhs, name);
                    }
                    cg!(self, "return size;");
                    self.dedent();
                    cg!(self, "}}");
                    cg!(self);
                },
                Choice(ref c) => {
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            for e in seq.elements() {
                                self.elem_setter(e, &class_name, true)?;
                                self.elem_getter(e, &class_name, true)?;
                            }
                        } else {
                            self.elem_setter(elem, &class_name, true)?;
                            self.elem_getter(elem, &class_name, true)?;
                        }
                    }
                    self.pack_choice(c, &class_name)?;
                    cg!(self);
                    self.read_choice(c, &class_name)?;
                    cg!(self);
                    cg!(self, "constexpr size_t {}::size() {{", class_name);
                    self.indent();
                    cg!(self, "return sizeof(data);");
                    self.dedent();
                    cg!(self, "}}");
                    cg!(self);
                },
                Empty => {}
            }
        }
        Ok(())
    }

    fn simple_type(&mut self, simple: &SimpleType, class_name: &str) -> Result<()> {
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name(), class_name)?
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn element(&mut self, _elem: &Element) -> Result<()> {
        Ok(())
    }

    fn elem_setter(&mut self, elem: &Element, class_name: &str, is_choice: bool) -> Result<()> {
        let reference = if elem.reference() { "&" } else { "" };
        use ::flat_ast::Occurs::*;
        let type_base = if elem.is_defined() {
            class_name.split("::").collect::<Vec<_>>()[0].to_owned() + "::" + elem.type_()
        } else {
            elem.type_().clone()
        };
        let type_ = if let Some(ref o) = elem.occurs() {
            match o {
                Unbounded => format!("std::vector<{}>", type_base),
                Num(n) => {
                    let n = if let Ok(_) = n.parse::<u32>() {
                        n.to_owned()
                    } else if elem.occur_is_defined() {
                        class_name.split("::").collect::<Vec<_>>()[0].to_owned() + "::" + &n
                    } else {
                        n.to_owned()
                    };
                    format!("std::array<{}, {}>", type_base, n)
                }
            }
        } else {
            type_base
        };
        cg!(self, "{0}& {0}::set_{1}(const {2}{3} {1}) {{", class_name, elem.name(), type_, reference);
        self.indent();
        if let Some(bitset) = elem.bitset() {
            let bits = elem.bits().unwrap();
            cg!(self, "for (size_t i = 0; i < {}; ++i) {{", bits);
            self.indent();
            cg!(self, "this->{}[i + {}] = {} & (1 << i);", bitset.name, bitset.start, elem.name());
            self.dedent();
            cg!(self, "}}");
        } else {
            cg!(self, "this->{1}{0} = {0};", elem.name(), if is_choice { "data." } else { "" });
        }
        cg!(self, "return *this;");
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        if let Some(ref o) = elem.occurs() {
            match o {
                Unbounded => {
                    cg!(self, "{0}& {0}::add_{1}(const {2}{3} {1}) {{", class_name, elem.name(), elem.type_(), reference);
                    self.indent();
                    cg!(self, "this->{0}.emplace_back({0});", elem.name());
                    cg!(self, "return *this;");
                    self.dedent();
                    cg!(self, "}}");
                    cg!(self);
                },
                Num(_) => {
                    cg!(self, "{0}& {0}::set_{1}(const {2}{3} {1}, size_t index) {{", class_name, elem.name(), elem.type_(), reference);
                    self.indent();
                    cg!(self, "this->{0}[index] = {0};", elem.name());
                    cg!(self, "return *this;");
                    self.dedent();
                    cg!(self, "}}");
                    cg!(self);
                }
            }
        }      
        Ok(())
    }

    fn elem_getter(&mut self, elem: &Element, class_name: &str, is_choice: bool) -> Result<()> {
        use ::flat_ast::Occurs::*;
        let reference = if elem.reference() { "&" } else { "" };
        let type_base = if elem.is_defined() {
            class_name.split("::").collect::<Vec<_>>()[0].to_owned() + "::" + elem.type_()
        } else {
            elem.type_().clone()
        };
        let type_ = if let Some(ref o) = elem.occurs() {
            match o {
                Unbounded => format!("std::vector<{}>", type_base),
                Num(n) => {
                    let n = if let Ok(_) = n.parse::<u32>() {
                        n.to_owned()
                    } else if elem.occur_is_defined() {
                        class_name.split("::").collect::<Vec<_>>()[0].to_owned() + "::" + &n
                    } else {
                        n.to_owned()
                    };
                    format!("std::array<{}, {}>", type_base, n)
                }
            }
        } else {
            type_base.clone()
        };
        let is_const = if elem.reference() { "const " } else { "" };
        cg!(self, "{4}{2}{3} {0}::get_{1}() const {{", class_name, elem.name(), type_, reference, is_const);
        self.indent();
        if let Some(bitset) = elem.bitset() {
            let bits = elem.bits().unwrap();
            cg!(self, "{} {}_tmp = 0;", elem.type_(), elem.name());
            cg!(self, "for (size_t i = 0; i < {}; ++i) {{", bits);
            self.indent();
            cg!(self, "{}_tmp |= (this->{}[i + {}] << i);", elem.name(), bitset.name, bitset.start);
            self.dedent();
            cg!(self, "}}");
            cg!(self, "return {}_tmp;", elem.name());
        } else {
            cg!(self, "return {1}{0};", elem.name(), if is_choice { "data." } else { "" });
        }
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        let is_const = if elem.reference() { "const " } else { "" };
        if elem.occurs().is_some() {
            cg!(self, "{}{}{} {}::get_{}(size_t index) const {{", is_const, type_base, reference, class_name, elem.name());
            self.indent();
            cg!(self, "return {}[index];", elem.name());
            self.dedent();
            cg!(self, "}}");
            cg!(self);
        }
        Ok(())
    }

    fn create(&mut self, packet: &Packet) -> Result<()> {
        let args = packet.contents().iter().map(|elem| {
            use self::PacketContent::*;
            match elem {
                Element(ref e) => match e.init() {
                    self::ElementInitValue::Create => {
                        let base = if e.is_defined() {
                            packet.class_name().to_owned() + "::"
                        } else {
                            "".to_owned()
                        };
                        if let Some(ref o) = e.occurs() {
                                use ::flat_ast::Occurs::*;
                                let mut use_base = true;
                                let t = match o {
                                    Unbounded => {
                                        use_base = false;
                                        format!("std::vector<{}>", e.type_())
                                    },
                                    Num(n) => {
                                        let n = if let Ok(_) = n.parse::<u32>() {
                                            n.to_owned()
                                        } else if e.occur_is_defined() {
                                            base.to_owned() + "::" + &n
                                        } else {
                                            n.to_owned()
                                        };
                                        format!("std::array<{}, {}>", e.type_(), n)
                                    }
                                };

                                if use_base {
                                    "const ".to_owned() + &base + &t + &format!("& {}, ", e.name())
                                } else {
                                    "const ".to_owned() + &t + &format!("& {}, ", e.name())
                                }
                            } else {
                                "const ".to_owned() + &base + e.type_() + &format!("& {}, ", e.name())
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
        cg!(self, "{0} {0}::create({1}) {{", packet.class_name(), args);
        self.indent();
        cg!(self, "{} packet;", packet.class_name());
        for content in packet.contents() {
            match content {
                self::PacketContent::Element(ref e) => match e.init() {
                    self::ElementInitValue::Create => { cg!(self, "packet.set_{}({});", e.name(), e.name()); },
                    _ => {}
                },
                _ => {}
            }
        }
        cg!(self, "return packet;");
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        cg!(self, "{0} {0}::create(const uint8_t* buffer) {{", packet.class_name());
        self.indent();
        cg!(self, "CRoseReader reader(buffer, CRosePacket::size(buffer));");
        cg!(self, "return {}(reader);", packet.class_name());
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        cg!(self, "std::unique_ptr<{0}> {0}::allocate(const uint8_t* buffer) {{", packet.class_name());
        self.indent();
        cg!(self, "CRoseReader reader(buffer, CRosePacket::size(buffer));");
        cg!(self, "return std::make_unique<{}>(reader);", packet.class_name());
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn pack(&mut self, packet: &Packet, iserialize: &HashSet<String>) -> Result<()> {
        if packet.contents().len() == 0 {
            cg!(self, "bool {}::pack(CRoseBasePolicy&) const {{", packet.class_name());
        } else {
            cg!(self, "bool {}::pack(CRoseBasePolicy& writer) const {{", packet.class_name());
        }
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(elem) => {
                    let base = if let Some(ref read_write) = elem.read_write() {
                        read_write.clone()
                    } else if let Some(ref enum_type) = elem.enum_type() {
                        enum_type.clone()
                    } else if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) {
                        "iserialize".to_owned()
                    } else {
                        clean_base(elem.type_())
                    };
                    if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded | Num(_) => {
                                if let Some(ref s) = elem.size_occurs() {
                                    self.write_if_else(&format!("!writer.set_{}({}.size())", s, elem.name()), &[
                                        "return false;"
                                    ], None)?;
                                }
                                cg!(self, "for (const auto& elem : {}) {{", elem.name());
                                self.indent();
                                self.write_if_else(&format!("!writer.set_{}(elem)", base), &[
                                        "return false;"
                                    ], None)?;
                                self.dedent();
                                cg!(self, "}}");
                            }
                        }
                    } else {
                        let name = if let Some(bitset) = elem.bitset() {
                            if bitset.start == 0 {
                                Some(&bitset.name)
                            } else {
                                None
                            }
                        } else {
                            Some(elem.name())
                        };
                        let base = if elem.bitset().is_some() {
                            "bitset".to_owned()
                        } else {
                            base
                        };
                        if let Some(name) = name {
                            self.write_if_else(&format!("!writer.set_{}({})", base, name), &[
                                    "return false;"
                                ], None)?;
                        }
                    }
                },
                _ => {}
            }
        }
        cg!(self, "return true;");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn pack_sequence(&mut self, packet: &Sequence, class_name: &str, iserialize: &HashSet<String>) -> Result<()> {
        cg!(self, "bool {}::write(CRoseBasePolicy& writer) const {{", class_name);
        self.indent();
        for elem in packet.elements() {
            let base = if let Some(ref enum_type) = elem.enum_type() {
                enum_type.clone()
            } else if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) {
                "iserialize".to_owned()
            } else {
                clean_base(elem.type_())
            };
            if let Some(ref o) = elem.occurs() {
                use ::flat_ast::Occurs::*;
                match o {
                    Unbounded => {
                        if let Some(ref s) = elem.size_occurs() {
                            self.write_if_else(&format!("!writer.set_{}({}.size())", s, elem.name()), &[
                                "return false;"
                            ], None)?;
                        }
                        cg!(self, "for (const auto& elem : {}) {{", elem.name());
                        self.indent();
                        self.write_if_else(&format!("!writer.set_{}(elem)", base), &[
                                "return false;"
                            ], None)?;
                        self.dedent();
                        cg!(self, "}}");
                    },
                    Num(n) => {
                        cg!(self, "for (size_t index = 0; index < {}; ++index) {{", n);
                        self.indent();
                        self.write_if_else(&format!("!writer.set_{}({}[index])", base, elem.name()), &[
                                "return false;"
                            ], None)?;
                        self.dedent();
                        cg!(self, "}}");
                    }
                }
            } else {
                let name = if let Some(bitset) = elem.bitset() {
                    if bitset.start == 0 {
                        Some(&bitset.name)
                    } else {
                        None
                    }
                } else {
                    Some(elem.name())
                };
                let base = if elem.bitset().is_some() {
                    "bitset".to_owned()
                } else {
                    base
                };
                if let Some(name) = name {
                    self.write_if_else(&format!("!writer.set_{}({})", base, name), &[
                            "return false;"
                        ], None)?;
                }
            }
        }
        cg!(self, "return true;");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn read_sequence(&mut self, packet: &Sequence, class_name: &str, iserialize: &HashSet<String>) -> Result<()> {
        cg!(self, "bool {}::read(CRoseReader& reader) {{", class_name);
        self.indent();
        for elem in packet.elements() {
            let base = if let Some(ref enum_type) = elem.enum_type() {
                enum_type.clone()
            } else if iserialize.contains(&elem.type_().to_owned().to_lower_camel_case()) {
                "iserialize".to_owned()
            } else {
                clean_base(elem.type_())
            };
            if let Some(ref o) = elem.occurs() {
                use ::flat_ast::Occurs::*;
                match o {
                    Unbounded => {
                        if let Some(ref s) = elem.size_occurs() {
                            self.write_if_else(&format!("!reader.get_{}({}.size())", s, elem.name()), &[
                                "return false;"
                            ], None)?;
                        }
                        cg!(self, "for (const auto& elem : {}) {{", elem.name());
                        self.indent();
                        self.write_if_else(&format!("!reader.get_{}(elem)", base), &[
                                "return false;"
                            ], None)?;
                        self.dedent();
                        cg!(self, "}}");
                    },
                    Num(n) => {
                        cg!(self, "for (size_t index = 0; index < {}; ++index) {{", n);
                        self.indent();
                        self.write_if_else(&format!("!reader.get_{}({}[index])", base, elem.name()), &[
                                "return false;"
                            ], None)?;
                        self.dedent();
                        cg!(self, "}}");
                    }
                }
            } else {
                let name = if let Some(bitset) = elem.bitset() {
                    if bitset.start == 0 {
                        Some(&bitset.name)
                    } else {
                        None
                    }
                } else {
                    Some(elem.name())
                };
                let base = if elem.bitset().is_some() {
                    "bitset".to_owned()
                } else {
                    base
                };
                if let Some(name) = name {
                    self.write_if_else(&format!("!reader.get_{}({})", base, name), &[
                            "return false;"
                        ], None)?;
                }
            }
        }
        cg!(self, "return true;");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn pack_choice(&mut self, packet: &Choice, class_name: &str) -> Result<()> {
        cg!(self, "bool {}::write(CRoseBasePolicy& writer) const {{", class_name);
        self.indent();
        let (max_size, member) = packet.elements().iter().fold((0, ""), |(size, member), elem| {
            let s = if elem.type_() == "uint8_t" {
                8
            } else if elem.type_() == "uint16_t" {
                16
            } else if elem.type_() == "uint32_t" || elem.type_() == "float" {
                32
            } else if elem.type_() == "uint64_t" || elem.type_() == "double" {
                64
            } else {
                debug!("type {} not recognized!", elem.type_());
                0
            };
            let s = if let Some(bits) = elem.bits() { s - bits } else { s };
            if size > s {
                (size, member)
            } else {
                (s, elem.name())
            }
        });
        let max_size = match max_size {
            8 => "uint8_t",
            16 => "uint16_t",
            32 => "uint32_t",
            64 => "uint64_t",
            _ => panic!("{} is not an expected size for an union!", max_size)
        };
        self.write_if_else(&format!("!writer.set_{}(data.{})", max_size, member), &[
                "return false;"
            ], None)?;
        cg!(self, "return true;");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn read_choice(&mut self, packet: &Choice, class_name: &str) -> Result<()> {
        cg!(self, "bool {}::read(CRoseReader& reader) {{", class_name);
        self.indent();
        let (max_size, member) = packet.elements().iter().fold((0, ""), |(size, member), elem| {
            let s = if elem.type_() == "uint8_t" {
                8
            } else if elem.type_() == "uint16_t" {
                16
            } else if elem.type_() == "uint32_t" || elem.type_() == "float" {
                32
            } else if elem.type_() == "uint64_t" || elem.type_() == "double" {
                64
            } else {
                0
            };
            let s = if let Some(bits) = elem.bits() { s - bits } else { s };
            if size > s {
                (size, member)
            } else {
                (s, elem.name())
            }
        });
        let max_size = match max_size {
            8 => "uint8_t",
            16 => "uint16_t",
            32 => "uint32_t",
            64 => "uint64_t",
            _ => panic!("Not a normal size for union!")
        };
        self.write_if_else(&format!("!reader.get_{}(data.{})", max_size, member), &[
                "return false;"
            ], None)?;
        cg!(self, "return true;");
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
                    cg!(self, "bool valid = true;");
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
                    cg!(self, "bool valid = true;");
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
            cg!(self);
            cg!(self, "constexpr size_t {}::{}::size() {{", class_name, name);
            self.indent();
            cg!(self, "size_t size = 0;");
            let mut tmp = false;
            for content in restrict.contents() {
                match content {
                    Length(l) => { tmp = true; cg!(self, "size += {}; // {}", l, data); },
                    _ => {}
                }
            }
            if !tmp {
                cg!(self, "size += sizeof({});", base);
            }
            cg!(self, "return size;");
            self.dedent();
            cg!(self, "}}");
            cg!(self);
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
