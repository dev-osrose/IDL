use ::flat_ast::*;
use std::io::{Result, Write};
use ::heck::*;
use std::collections::HashMap;
use flat_ast::RestrictionContent::{Enumeration, Length, MaxValue, MinValue};

pub (crate) struct CodeSourceGenerator<'a, W: Write + 'a> {
    writer: &'a mut ::writer::Writer<W>,
    version: String
}

impl<'a, W: Write> CodeSourceGenerator<'a, W> {
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
        let version = self.version.clone();
        cg!(self, "/* This file is @generated with IDL v{} */\n", version);
        cg!(self, r#"use bincode::{{Encode, Decode, enc::Encoder, de::Decoder, error::DecodeError}};"#);
        cg!(self, r#"use bincode::de::read::Reader;"#);
        cg!(self, r#"use bincode::enc::write::Writer;"#);
        cg!(self, r#"use utils::null_string::NullTerminatedString;"#);
        cg!(self, r#"use crate::enums::*;"#);
        cg!(self, r#"use crate::types::*;"#);
        cg!(self, r#"use crate::dataconsts::*;"#);
        cg!(self, r#"use crate::packet::PacketPayload;"#);

        let mut iserialize: HashMap<String, String> = HashMap::new();
        iserialize.insert("int8_t".to_string(), "i8".to_string());
        iserialize.insert("uint8_t".to_string(), "u8".to_string());
        iserialize.insert("int16_t".to_string(), "i16".to_string());
        iserialize.insert("uint16_t".to_string(), "u16".to_string());
        iserialize.insert("int32_t".to_string(), "i32".to_string());
        iserialize.insert("uint32_t".to_string(), "u32".to_string());
        iserialize.insert("int64_t".to_string(), "i64".to_string());
        iserialize.insert("uint64_t".to_string(), "u64".to_string());
        iserialize.insert("char".to_string(), "u8".to_string());
        iserialize.insert("int".to_string(), "i32".to_string());
        iserialize.insert("unsigned int".to_string(), "u32".to_string());
        iserialize.insert("float".to_string(), "f32".to_string());
        iserialize.insert("double".to_string(), "f64".to_string());
        iserialize.insert("std::string".to_string(), "NullTerminatedString".to_string());


        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Simple(simple) => self.simple_type(simple, &iserialize)?,
                _ => {}
            }
        }
        cg!(self);
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Complex(ref complex) => self.complex_type(complex, &iserialize)?,
                _ => {}
            };
        }

        cg!(self);

        cg!(self, r#"#[derive(Debug)]"#);
        cg!(self, "pub struct {} {{", packet.class_name().to_upper_camel_case());
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => self.element(elem, &iserialize)?,
                _ => {}
            };
        }
        self.dedent();
        cg!(self, "}}");

        cg!(self);
        cg!(self, "impl PacketPayload for {} {{}}", packet.class_name().to_upper_camel_case());
        cg!(self);
        cg!(self, "impl Encode for {} {{", packet.class_name().to_upper_camel_case());
        self.indent();
        cg!(self, "fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), bincode::error::EncodeError> {{");
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => {
                    let name = rename_if_reserved(elem.name());
                    let trimmed_type = elem.type_().trim().to_string();
                    let mut is_rust_native = true;
                    let rust_type = iserialize.get(elem.type_().trim()).unwrap_or_else(|| {
                        debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
                        is_rust_native = false;
                        &trimmed_type
                    });

                    if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded => {
                                cg!(self, "self.{}.encode(encoder)?;", name);
                            }
                            Num(n) => {
                                cg!(self, "for value in &self.{} {{", name);
                                self.indent();
                                cg!(self, "value.encode(encoder)?;");
                                self.dedent();
                                cg!(self, "}}");
                            }
                        };
                    } else {
                        cg!(self, "self.{}.encode(encoder)?;", name);
                    }
                },
                _ => {}
            };
        }
        cg!(self, "Ok(())");
        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");

        cg!(self);
        cg!(self, "impl Decode for {} {{", packet.class_name().to_upper_camel_case());
        self.indent();
        cg!(self, "fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, bincode::error::DecodeError> {{");
        self.indent();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => {
                    let name = rename_if_reserved(elem.name());
                    let trimmed_type = elem.type_().trim().to_string();
                    let mut is_rust_native = true;
                    let rust_type = iserialize.get(elem.type_().trim()).unwrap_or_else(|| {
                        debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
                        is_rust_native = false;
                        &trimmed_type
                    });
                    if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded => {
                                cg!(self, "let {} = Vec::decode(decoder)?;", name);
                            }
                            Num(n) => {
                                let mut type_prefix = "0";
                                if "String" == rust_type {
                                    type_prefix = "";
                                }

                                if false == is_rust_native {
                                    cg!(self, "let mut {} = Vec::with_capacity({} as usize);", name, n);
                                    cg!(self, "for _ in 0..{} as usize {{", n);
                                    self.indent();
                                    cg!(self, "{}.push({}::decode(decoder)?);", name, rust_type);
                                    self.dedent();
                                    cg!(self, "}}");
                                } else {
                                    if n.parse::<usize>().is_ok() {
                                        cg!(self, "let mut {} = [{}{}; {}];", name, type_prefix, rust_type, n);
                                    } else {
                                        cg!(self, "let mut {} = [{}{}; ({} as usize)];", name, type_prefix, rust_type, n);
                                    }
                                    cg!(self, "for value in &mut {} {{", name);
                                    self.indent();
                                    cg!(self, "*value = {}::decode(decoder)?;", rust_type);
                                    self.dedent();
                                    cg!(self, "}}");
                                }
                            }
                        };
                    } else {
                        cg!(self, "let {} = {}::decode(decoder)?;", name, rust_type.to_owned().to_string());
                    };
                },
                _ => {}
            };
        }
        let mut output_list = Vec::new();
        for content in packet.contents() {
            use self::PacketContent::*;
            match content {
                Element(ref elem) => {
                    output_list.push(rename_if_reserved(elem.name()));
                },
                _ => {}
            };
        }

        cg!(self, "Ok(Self {{ {} }})", output_list.join(", "));
        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");


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

    fn get_bitfield_type(bit_count: usize) -> &'static str {
        match bit_count {
            1..=8 => "u8",
            9..=16 => "u16",
            17..=32 => "u32",
            33..=64 => "u64",
            _ => panic!("Unsupported bit count: {}", bit_count),
        }
    }

    fn get_size_of_type(type_name: &str) -> usize {
        match type_name {
            "u8" => std::mem::size_of::<u8>(),
            "u16" => std::mem::size_of::<u16>(),
            "u32" => std::mem::size_of::<u32>(),
            "u64" => std::mem::size_of::<u64>(),
            _ => panic!("Unsupported type: {}", type_name),
        }
    }

    fn complex_type(&mut self, complex: &ComplexType, iserialize: &HashMap<String, String>) -> Result<()> {
        use ::flat_ast::ComplexTypeContent::*;
        if complex.inline() == false {
            // All unions need to be outside the struct
            match complex.content() {
                Choice(ref c) => {
                    for elem in c.elements() {
                        if let Some(ref seq) = c.inline_seqs().get(elem.name()) {
                            cg!(self, r#"#[derive(Debug)]"#);
                            cg!(self, "struct {} {{", elem.name());
                            self.indent();
                            for e in seq.elements() {
                                self.element(e, &iserialize)?;
                            }
                            self.dedent();
                            cg!(self, "}}");

                            // Get the max size of the union
                            let mut max_bit_size = 0;

                            for elem2 in c.elements() {
                                if let Some(ref seq2) = c.inline_seqs().get(elem2.name()) {
                                    // Do nothing
                                } else {
                                    let trimmed_type = elem2.type_().trim().to_string();
                                    let rust_type = iserialize.get(elem2.type_().trim()).map(|s| s.to_string()).unwrap_or_else(|| {
                                        warn!(r#"Type "{}" not found, outputting anyway"#, elem2.type_());
                                        trimmed_type.clone()
                                    });
                                    max_bit_size = Self::get_size_of_type(&rust_type) as u32 * 8;
                                }
                            }
                            let rust_type = Self::get_bitfield_type(max_bit_size as usize);

                            cg!(self);
                            cg!(self, "impl {} {{", elem.name());
                            self.indent();
                            cg!(self, "fn encode_bitfield(value: {}, size: {}, offset: &mut {}) -> {} {{", rust_type, rust_type, rust_type, rust_type);
                            self.indent();
                            cg!(self, "let encoded = (value & ((1 << size) - 1)) << *offset;");
                            cg!(self, "*offset += size; // Update offset for the next field");
                            cg!(self, "encoded");
                            self.dedent();
                            cg!(self, "}}");
                            cg!(self);
                            cg!(self, "fn decode_bitfield(encoded: {}, size: {}, offset: &mut {}) -> {} {{", rust_type, rust_type, rust_type, rust_type);
                            self.indent();
                            cg!(self, "let value = (encoded >> *offset) & ((1 << size) - 1);");
                            cg!(self, "*offset += size; // Update offset for the next field");
                            cg!(self, "value");
                            self.dedent();
                            cg!(self, "}}");
                            cg!(self);
                            cg!(self, "pub fn encode_data(&self) -> {} {{", rust_type);
                            self.indent();
                            cg!(self, "let mut offset = 0;");
                            let mut variable_names = Vec::new();
                            for e in seq.elements() {
                                let name = rename_if_reserved(e.name());
                                let bits = e.bits().map_or_else(|| "".to_string(), |b| format!("{}", b));
                                cg!(self, "let {}_bits = Self::encode_bitfield(self.{}, {}, &mut offset);", e.name().to_snake_case(), name, bits);
                                variable_names.push(format!("{}_bits", e.name().to_snake_case()));
                            }
                            cg!(self, "{}", variable_names.join(" | "));
                            self.dedent();
                            cg!(self, "}}");
                            self.dedent();
                            cg!(self, "}}");
                            cg!(self);

                            cg!(self, "impl Encode for {} {{", elem.name());
                            self.indent();
                            cg!(self, "fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), bincode::error::EncodeError> {{");
                            self.indent();
                            cg!(self, "self.encode_data().encode(encoder)?;");
                            cg!(self, "Ok(())");
                            self.dedent();
                            cg!(self, "}}");
                            self.dedent();
                            cg!(self, "}}");

                            cg!(self);
                            cg!(self, "impl Decode for {} {{", elem.name());
                            self.indent();
                            cg!(self, "fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, bincode::error::DecodeError> {{");
                            self.indent();
                            cg!(self, "let bitfield = {}::decode(decoder)?;", rust_type);
                            cg!(self, "let mut offset = 0;");
                            let mut variable_names = Vec::new();
                            for e in seq.elements() {
                                let name = rename_if_reserved(e.name());
                                let bits = e.bits().map_or_else(|| "".to_string(), |b| format!("{}", b));
                                cg!(self, "let {} = Self::decode_bitfield(bitfield, {}, &mut offset);", name, bits);
                                variable_names.push(format!("{}", name));
                            }
                            cg!(self, "Ok(Self {{ {} }})", variable_names.join(", "));
                            self.dedent();
                            cg!(self, "}}");
                            self.dedent();
                            cg!(self, "}}");
                        }
                    }
                },
                _ => {}
            }

            cg!(self);
            cg!(self, r#"#[derive(Debug)]"#);
            cg!(self, "pub struct {} {{", complex.name());
            self.indent();
            match complex.content() {
                Seq(ref s) => {
                    for elem in s.elements() {
                        self.element(elem, &iserialize)?;
                    }
                },
                Choice(ref c) => {
                    for elem in c.elements() {
                        if let Some(ref _seq) = c.inline_seqs().get(elem.name()) {
                            cg!(self, "{}: {},", elem.name().to_snake_case(), elem.name());
                        } else {
                            self.element(elem, &iserialize)?;
                        }
                    }
                },
                Empty => {}
            }
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            let _ = self.complex_encode(complex, iserialize);
            cg!(self);
            let _ = self.complex_decode(complex, iserialize);
        }
        Ok(())
    }

    fn complex_encode(&mut self, complex: &ComplexType, _iserialize: &HashMap<String, String>) -> Result<()> {
        use ::flat_ast::ComplexTypeContent::*;
        cg!(self, "impl Encode for {} {{", complex.name());
        self.indent();
        cg!(self, "fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), bincode::error::EncodeError> {{");
        self.indent();

        match complex.content() {
            Seq(ref s) => {
                for elem in s.elements() {
                    let data = elem.name().to_string().to_snake_case();
                    cg!(self, "self.{}.encode(encoder)?;", data);
                }
            },
            Choice(ref c) => {
                for elem in c.elements() {
                    let data = elem.name().to_string().to_snake_case();
                    cg!(self, "self.{}.encode(encoder)?;", data);
                }
            },
            Empty => {}
        }
        cg!(self, "Ok(())");

        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }

    fn complex_decode(&mut self, complex: &ComplexType, iserialize: &HashMap<String, String>) -> Result<()> {
        use ::flat_ast::ComplexTypeContent::*;
        cg!(self, "impl Decode for {} {{", complex.name().to_upper_camel_case());
        self.indent();
        cg!(self, "fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, bincode::error::DecodeError> {{");
        self.indent();

        let mut output_list = Vec::new();
        match complex.content() {
            Seq(ref s) => {
                for elem in s.elements() {
                    let name = elem.name().to_string().to_snake_case();
                    let trimmed_type = elem.type_().trim().to_string();
                    let mut is_rust_native = true;
                    let rust_type = iserialize.get(elem.type_().trim()).map(|s| s.to_string()).unwrap_or_else(|| {
                        debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
                        is_rust_native = false;
                        trimmed_type.clone()
                    });

                    if let Some(ref o) = elem.occurs() {
                        use ::flat_ast::Occurs::*;
                        match o {
                            Unbounded => {
                                cg!(self, "let {} = Vec::decode(decoder)?;", name);
                            }
                            Num(n) => {
                                let mut type_prefix = "0";
                                if "String" == rust_type {
                                    type_prefix = "";
                                }

                                if false == is_rust_native {
                                    cg!(self, "let mut {} = Vec::with_capacity({} as usize);", name, n);
                                    cg!(self, "for _ in 0..{} as usize {{", n);
                                    self.indent();
                                    cg!(self, "{}.push({}::decode(decoder)?);", name, rust_type);
                                    self.dedent();
                                    cg!(self, "}}");
                                } else {
                                    if n.parse::<usize>().is_ok() {
                                        cg!(self, "let mut {} = [{}{}; {}];", name, type_prefix, rust_type, n);
                                    } else {
                                        cg!(self, "let mut {} = [{}{}; ({} as usize)];", name, type_prefix, rust_type, n);
                                    }
                                    cg!(self, "for value in &mut {} {{", name);
                                    self.indent();
                                    cg!(self, "*value = {}::decode(decoder)?;", rust_type);
                                    self.dedent();
                                    cg!(self, "}}");
                                }
                            }
                        };
                    } else {
                        cg!(self, "let {} = {}::decode(decoder)?;", name, rust_type);
                    }
                    output_list.push(name);
                }
            },
            Choice(ref c) => {
                for elem in c.elements() {
                    let name = elem.name().to_string().to_snake_case();
                    let trimmed_type = elem.type_().trim().to_string();
                    let rust_type = iserialize.get(elem.type_().trim()).map(|s| s.to_string()).unwrap_or_else(|| {
                        debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
                        trimmed_type.clone()
                    });

                    cg!(self, "let {} = {}::decode(decoder)?;", name, rust_type);
                    output_list.push(name);
                }
            },
            Empty => {}
        }
        cg!(self, "Ok(Self {{ {} }})", output_list.join(", "));

        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");
        Ok(())
    }


    fn simple_type(&mut self, simple: &SimpleType, iserialize: &HashMap<String, String>) -> Result<()> {
        cg!(self);
        self.doc(simple.doc())?;
        for content in simple.contents() {
            match content {
                SimpleTypeContent::Restriction(res) => self.restrict(res, simple.name(), &iserialize)?
            }
        }
        Ok(())
    }

    fn element(&mut self, elem: &Element, iserialize: &HashMap<String, String>) -> Result<()> {
        self.doc(elem.doc())?;

        if let Some(bitset) = elem.bitset() {
            if bitset.start == 0 {
                cg!(self, "{}: [bool; {}],", bitset.name, bitset.size);
            }
            return Ok(());
        }

        let trimmed_type = elem.type_().trim().to_string();
        let mut is_rust_native = true;
        let rust_type = iserialize.get(elem.type_().trim()).unwrap_or_else(|| {
            debug!(r#"Type "{}" not found, outputting anyway"#, elem.type_());
            is_rust_native = false;
            &trimmed_type
        });

        let (type_, bits) = if let Some(ref o) = elem.occurs() {
            use ::flat_ast::Occurs::*;
            let type_ = match o {
                Unbounded => format!("Vec<{}>", rust_type),
                Num(n) => {
                    if false == is_rust_native {
                        format!("Vec<{}>", rust_type)
                    } else {
                        if n.parse::<usize>().is_ok() {
                            format!("[{}; {}]", rust_type, n)
                        } else {
                            format!("[{}; ({} as usize)]", rust_type, n)
                        }
                    }
                }
            };
            (type_, "".to_string())
        } else {
            let bits = elem.bits().map_or_else(|| "".to_string(), |b| format!("// {} bits", b));
            (rust_type.to_owned().to_string(), bits)
        };
        // let default = match elem.init() {
        //     self::ElementInitValue::Default(d) => " = ".to_string() + d,
        //     _ => "".to_string()
        // };
        let name = rename_if_reserved(elem.name());
        // cg!(self, "{}: {}{}{},", elem.name(), type_, bits, default);
        cg!(self, "pub(crate) {}: {}, {}", name, type_, bits);
        Ok(())
    }

    fn restrict(&mut self, restrict: &Restriction, name: &str, iserialize: &HashMap<String, String>) -> Result<()> {
        use self::RestrictionContent::*;
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();
        self.doc(restrict.doc())?;
        let base = restrict.base().trim().to_string();
        let mut rust_type = iserialize.get(restrict.base().trim()).map(|s| s.to_string()).unwrap_or_else(|| {
            debug!(r#"Type "{}" not found, outputting anyway"#, base);
            base.clone()
        });

        if "NullTerminatedString" == rust_type {
            rust_type = "String".to_string();
        }

        if is_enum {
            cg!(self, r#"#[repr({})]"#, rust_type);
            cg!(self, r#"#[derive(Debug, Clone)]"#);
            cg!(self, "pub(crate) enum {} {{", name.to_upper_camel_case());
            self.indent();
            for content in restrict.contents() {
                if let Enumeration(en) = content {
                    self.doc(en.doc())?;
                    cg!(self, "{} = {},", en.value().to_upper_camel_case(), en.id());
                }
            }
        } else {
            cg!(self, r#"#[derive(Debug)]"#);
            cg!(self, "pub struct {} {{", name.to_upper_camel_case());
            self.indent();
            cg!(self, "pub(crate) {}: {},", name.to_string().to_snake_case(), rust_type);
        }

        self.dedent();
        cg!(self, "}}");

        cg!(self);
        self.restrict_encode(&restrict, name, iserialize)?;
        cg!(self);
        self.restrict_decode(&restrict, name, iserialize)?;
        Ok(())
    }

    fn restrict_encode(&mut self, restrict: &Restriction, name: &str, _iserialize: &HashMap<String, String>) -> Result<()> {
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();

        cg!(self, "impl Encode for {} {{", name.to_upper_camel_case());
        self.indent();
        cg!(self, "fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), bincode::error::EncodeError> {{");
        self.indent();
        if is_enum {
            cg!(self, "encoder.writer().write(&[self.clone() as u8]).map_err(Into::into)");
        } else {
            let data = name.to_string().to_snake_case();
            cg!(self, "let bytes = self.{}.as_bytes();", data);
            for content in restrict.contents() {
                match content {
                    Length(l) => {
                        cg!(self, "let fixed_length = {};", l);
                        cg!(self, "if bytes.len() > fixed_length {{");
                        self.indent();
                        cg!(self, "return Err(bincode::error::EncodeError::OtherString(format!(");
                        cg!(self, "\"{} length exceeds fixed size: {{}} > {{}}\", bytes.len(), fixed_length)));", data);
                        self.dedent();
                        cg!(self, "}}");
                        cg!(self, "encoder.writer().write(bytes)?;");
                        cg!(self, "encoder.writer().write(&vec![0; fixed_length - bytes.len()])?;");
                        cg!(self, "Ok(())");
                    },
                    MinValue(_v) => {

                    },
                    MaxValue(_v) => {

                    },
                    _ => panic!("enumeration in restrict when there shouldn't be one")
                }
            }
        }
        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");

        Ok(())
    }

    fn restrict_decode(&mut self, restrict: &Restriction, name: &str, iserialize: &HashMap<String, String>) -> Result<()> {
        let is_enum = restrict.contents().iter().find(|content| match content {
            Enumeration(_) => true,
            _ => false
        }).is_some();
        let trimmed_type = restrict.base().trim().to_string();
        let mut rust_type = iserialize.get(restrict.base().trim()).map(|s| s.to_string()).unwrap_or_else(|| {
            debug!(r#"Type "{}" not found, outputting anyway"#, restrict.base());
            trimmed_type.clone()
        });

        if "NullTerminatedString" == rust_type {
            rust_type = "String".to_string();
        }

        cg!(self, "impl Decode for {} {{", name.to_upper_camel_case());
        self.indent();
        cg!(self, "fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, bincode::error::DecodeError> {{");
        self.indent();
        if is_enum {
            cg!(self, "let value = {}::decode(decoder)?;", rust_type);
            cg!(self, "match value {{");
            self.indent();
            for content in restrict.contents() {
                if let Enumeration(en) = content {
                    cg!(self, "{} => Ok({}::{}),", en.id(), name.to_upper_camel_case(), en.value().to_upper_camel_case());
                }
            }
            cg!(self, "_ => Err(bincode::error::DecodeError::OtherString(format!(\"Invalid value for {}: {{}}\", value))),", name.to_upper_camel_case());
            self.dedent();
            cg!(self, "}}");
        } else {
            let data = name.to_string().to_snake_case();
            let mut fixed_length = 64;
            let mut min_value_check = String::new();
            let mut max_value_check = String::new();
            for content in restrict.contents() {
                match content {
                    Length(l) => {
                        fixed_length = *l;
                    },
                    MinValue(v) => {
                        min_value_check = format!("if {} < {} {{Err(bincode::error::DecodeError::OtherString(format!(\"Invalid value for {}: {{}} < {{}}\", {}, {})))}}", data, v, data, data, v).into();
                    },
                    MaxValue(v) => {
                        max_value_check = format!("if {} > {} {{Err(bincode::error::DecodeError::OtherString(format!(\"Invalid value for {}: {{}} > {{}}\", {}, {})))}}", data, v, data, data, v).into();
                    },
                    _ => panic!("enumeration in restrict when there shouldn't be one")
                }
            }

            if rust_type == "String" {
                cg!(self, "let mut buffer = vec![0u8; {}];", fixed_length);
                cg!(self, "decoder.reader().read(&mut buffer)?;");
                cg!(self, "let {} = {}::from_utf8(buffer)", data, rust_type);
                cg!(self, ".map_err(|e| DecodeError::OtherString(format!(\"Invalid UTF-8: {{}}\", e)))?");
                cg!(self, ".trim_end_matches('\\0')");
                cg!(self, ".to_string();");
            } else {
                cg!(self, "let {} = {}::decode(buffer)?;", data, rust_type);

                cg!(self, "{}", min_value_check);

                cg!(self, "{}", max_value_check);
            }
            cg!(self, "Ok(Self {{ {} }})", data);
        }
        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");

        Ok(())
    }
}

fn rename_if_reserved(name: &str) -> String {
    let reserved_keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
        "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
        "abstract", "become", "box", "do", "final", "macro", "override", "priv",
        "try", "typeof", "unsized", "virtual", "yield",
    ];

    if reserved_keywords.contains(&name) {
        format!("{}_", name.to_snake_case()) // Append a suffix to avoid conflicts
    } else {
        name.to_string().to_snake_case()
    }
}
