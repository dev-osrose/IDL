use ::packet_schema::ast;
use ::flat_ast;
use ::packet_schema::Reader;
use std::fs::File;
use std::collections::HashSet;

struct Context<'a> {
    packet: &'a mut flat_ast::Packet,
    path: Vec<String>,
    complex_types: HashSet<String>,
    is_in_choice: bool,
    bitsets: u32,
    current_bitset: Option<u32>
}

impl<'a> Context<'a> {
    fn add_content(&mut self, content: flat_ast::PacketContent) {
        if let flat_ast::PacketContent::Complex(ref cot) = content {
            self.complex_types.insert(cot.name().clone());
        }
        self.packet.add_content(content);
    }

    fn get_anon_name(&self) -> String {
        let name = self.path.iter().fold("".to_string(), |path, curr| path + curr);
        if !self.complex_types.contains(&name) {
            name
        } else {
            let mut i = 0;
            loop {
                i += 1;
                let name = format!("{}{}", name, i);
                if !self.complex_types.contains(&name) {
                    break name
                }
            }
        }
    }

    fn ref_match(ref_name: &String, elem_name: &String) -> bool {
        let idx = match ref_name.find(":") {
            Some(idx) => idx + 1,
            _ => 0
        };
        &ref_name[idx..] == &elem_name[..]
    }

    fn find_ref(&self, name: &String) -> Option<&flat_ast::Element> {
        for content in self.packet.contents() {
            use self::flat_ast::PacketContent::Element;
            match content {
                Element(e) if Self::ref_match(name, e.name()) => return Some(e),
                _ => {}
            }
        }
        None
    }

    fn find_bitset_mut_ref(&mut self, id: u32) -> Vec<&mut flat_ast::Element> {
        let mut res = Vec::new();
        let id = format!("bitset{}", id);
        for content in self.packet.contents_mut() {
            use self::flat_ast::PacketContent::{Element, Complex};
            match content {
                Element(e) => {
                    if let Some(bitset) = e.bitset() {
                        if bitset.name == id {
                            res.push(e);
                        }
                    }
                },
                Complex(c) => {
                    use self::flat_ast::ComplexTypeContent::Seq;
                    match c.content_mut() {
                        Seq(s) => {
                            for elem in s.elements_mut() {
                                if let Some(bitset) = elem.bitset() {
                                    if bitset.name == id {
                                        res.push(elem);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        res
    }

    fn add_bits(&mut self, bits: u32) -> Option<u32> {
        if self.is_in_choice {
            return None;
        }
        if let Some(ref mut bitset) = self.current_bitset {
            let old = *bitset;
            *bitset += bits;
            return if *bitset < 64 {
                Some(old)
            } else {
                panic!("Error, cannot have more than 64 bits bitfields in a row!");
            };
        }
        self.current_bitset = Some(bits);
        self.bitsets += 1;
        Some(0)
    }

    fn stop_bits(&mut self) {
        if let Some(bitset) = self.current_bitset {
            if bitset % 8 != 0 {
                panic!(format!("Error, {} bits cannot be aligned", bitset));
            }
            trace!("generating bitset of {} bits", bitset);
            for elem in self.find_bitset_mut_ref(self.bitsets) {
                trace!("{}", elem.name());
                elem.bitset_mut().as_mut().unwrap().size = bitset;
            }
            self.current_bitset = None;
        }
    }
}

pub fn flatten(search_path: &::std::path::Path, p: &ast::Packet) -> Result<flat_ast::Packet, ::failure::Error> {
    let mut packet = flat_ast::Packet::new(p.type_().clone(), p.doc().clone());
    {
        let mut ctx = Context {
            packet: &mut packet,
            path: Vec::new(),
            complex_types: HashSet::new(),
            is_in_choice: false,
            bitsets: 0,
            current_bitset: None
        };
        flatten_(search_path, p, &mut ctx)?;
        if ctx.bitsets != 0 {
            ctx.add_content(flat_ast::PacketContent::Include("bitset".to_owned(), true));
        }
    }
    Ok(packet)
}

fn flatten_(search_path: &::std::path::Path, packet: &ast::Packet, ctx: &mut Context) -> Result<(), ::failure::Error> {
    for content in packet.contents() {
        use flat_ast::PacketContent::*;
        match content {
            ast::PacketContent::Include(ref path, system) => {
                ctx.add_content(Include(path.clone(), *system));
            },
            ast::PacketContent::IncludeXml(ref location) => {
                let filenm = search_path.join(::std::path::Path::new(location));
                debug!("Including {}", filenm.to_str().unwrap());
                let file = File::open(&filenm)?;
                let packet = Reader::load_packet(file)?;
                flatten_(search_path, &packet, ctx)?;
            },
            ast::PacketContent::SimpleType(ref simple) => {
                let simple = flatten_simple(simple);
                ctx.add_content(Simple(simple));
            },
            ast::PacketContent::ComplexType(ref complex) => {
                ctx.path = vec![complex.name().clone()];
                let _complex = flatten_complex(complex, ctx);
            },
            ast::PacketContent::Element(ref element) => {
                let element = flatten_element(element, ctx, 0);
                ctx.add_content(Element(element));
            }
        }
    }
    ctx.stop_bits();
    Ok(())
}

fn flatten_simple(simple: &ast::SimpleType) -> flat_ast::SimpleType {
    let mut type_ = flat_ast::SimpleType::new(simple.name().clone(), simple.doc().clone());
    let mut enum_id = 0i64;
    for content in simple.contents() {
        match content {
            ast::SimpleTypeContent::Restriction(ref restriction) => {
                let restrict = flatten_restriction(restriction, &mut enum_id);
                type_.add_content(flat_ast::SimpleTypeContent::Restriction(restrict));
            }
        }
    }
    type_
}

fn flatten_restriction(r: &ast::Restriction, enum_id: &mut i64) -> flat_ast::Restriction {
    let mut res = flat_ast::Restriction::new(r.base().clone(), r.doc().clone());
    use self::ast::RestrictionContent::*;
    for content in r.contents() {
        let content = match content {
            Enumeration(ref e) => {
                let enm = flatten_enum(e, enum_id);
                flat_ast::RestrictionContent::Enumeration(enm)
            },
            Length(ref v) => flat_ast::RestrictionContent::Length(*v),
            MinValue(ref v) => flat_ast::RestrictionContent::MinValue(v.clone()),
            MaxValue(ref v) => flat_ast::RestrictionContent::MaxValue(v.clone())
        };
        res.add_content(content);
    }
    res
}

fn flatten_enum(e: &ast::Enumeration, enum_id: &mut i64) -> flat_ast::Enumeration {
    *enum_id = match e.id() {
        Some(val) => *val,
        None => *enum_id
    };
    *enum_id = *enum_id + 1;
    flat_ast::Enumeration::new(e.value().clone(), *enum_id - 1, e.doc().clone())
}

fn flatten_complex(c: &ast::ComplexType, ctx: &mut Context) {
    use flat_ast::ComplexTypeContent::*;
    use self::ast::ComplexTypeContent;
    let mut inline = false;
    let content = match c.content() {
        ComplexTypeContent::Choice(ref c) => Choice(flatten_choice(c, ctx)),
        ComplexTypeContent::Seq(ref s) => {
            let seq = flatten_seq(s, ctx);
            inline = seq.inline();
            Seq(seq)
        },
        ComplexTypeContent::Empty => Empty
    };
    let cot = flat_ast::ComplexType::new(c.name().clone(), content, c.doc().clone(), false, inline);
    ctx.add_content(flat_ast::PacketContent::Complex(cot));
    ctx.stop_bits();
}

fn flatten_anon_complex(c: &ast::AnonComplexType, ctx: &mut Context, element_name: &Option<String>) -> flat_ast::ComplexType {
    use flat_ast::ComplexTypeContent::*;
    use self::ast::ComplexTypeContent;
    let path = if let Some(ref name) = element_name {
        let mut itr = name.chars();
        let mut name = String::with_capacity(name.len());
        if let Some(ch) = itr.next() {
            name.extend(ch.to_uppercase());
        }
        while let Some(ch) = itr.next() {
            name.push(ch);
        }
        name
    } else {
        let path = match c.content() {
            ComplexTypeContent::Choice(_) => "Choice",
            ComplexTypeContent::Seq(_) => "Sequence",
            ComplexTypeContent::Empty => ""
        };
        path.to_string()
    };
    ctx.path.push(path);
    let mut inline = false;
    let content = match c.content() {
        ComplexTypeContent::Choice(ref c) => Choice(flatten_choice(c, ctx)),
        ComplexTypeContent::Seq(ref s) => {
            let seq = flatten_seq(s, ctx);
            inline = seq.inline();
            Seq(seq)
        },
        ComplexTypeContent::Empty => Empty
    };
    let name = ctx.get_anon_name();
    ctx.path.pop();
    flat_ast::ComplexType::new(name, content, c.doc().clone(), true, inline)
}

fn flatten_seq(s: &ast::Sequence, ctx: &mut Context) -> flat_ast::Sequence {
    let mut seq = flat_ast::Sequence::new(s.occurs().clone(), s.size_occurs().clone(), s.doc().clone(), s.inline());
    let mut max_id = 0;
    for content in s.contents() {
        let element = flatten_seq_content(content, ctx, max_id);
        if max_id <= element.id() {
            max_id = element.id() + 1;
        }
        seq.add_element(element);
    }
    seq
}

fn flatten_choice(c: &ast::Choice, ctx: &mut Context) -> flat_ast::Choice {
    let mut choice = flat_ast::Choice::new(c.occurs().clone(), c.size_occurs().clone(), c.doc().clone());
    let mut max_id = 0;
    ctx.is_in_choice = true;
    for content in c.contents() {
        let element = flatten_seq_content(content, ctx, max_id);
        if max_id <= element.id() {
            max_id = element.id() + 1;
        }
        choice.add_element(element);
    }
    ctx.is_in_choice = false;
    choice
}

fn flatten_seq_content(c: &ast::SequenceContent, ctx: &mut Context, id: u32) -> flat_ast::Element {
    let (name, occurs, size_occurs, doc, content, inline) = match c {
        ast::SequenceContent::Element(ref element) => {
            return flatten_element(element, ctx, id);
        },
        ast::SequenceContent::Choice(ref choice) => {
            ctx.path.push("Choice".to_string());
            let choice = flatten_choice(choice, ctx);
            let doc = choice.doc().clone();
            let occurs = choice.occurs().clone();
            let size_occurs = choice.size_occurs().clone();
            let name = ctx.get_anon_name();
            ctx.path.pop();
            let content = flat_ast::ComplexTypeContent::Choice(choice);
            (name, occurs, size_occurs, doc, content, false)
        },
        ast::SequenceContent::Seq(ref seq) => {
            ctx.path.push("Sequence".to_string());
            let seq = flatten_seq(seq, ctx);
            let inline = seq.inline();
            let occurs = seq.occurs().clone();
            let size_occurs = seq.size_occurs().clone();
            let doc = seq.doc().clone();
            let name = ctx.get_anon_name();
            ctx.path.pop();
            let content = flat_ast::ComplexTypeContent::Seq(seq);
            (name, occurs, size_occurs, doc, content, inline)
        }
    };

    let complex = flat_ast::ComplexType::new(name.clone(), content, doc.clone(), true, inline);
    ctx.add_content(flat_ast::PacketContent::Complex(complex));
    flat_ast::Element::new(name.clone(), name.clone(), id,
        flat_ast::ElementInitValue::None, occurs, size_occurs, doc, true, true, None, None, None)
}

fn flatten_element(elem: &ast::Element, ctx: &mut Context, id: u32) -> flat_ast::Element {
    let init = elem.init().clone();
    let (name, type_, anonymous) = match elem.type_() {
        ast::ElementType::Named{ ref name, ref type_ } => (name.clone(), type_.clone(), false),
        ast::ElementType::Ref(ref name) => {
            if let Some(elem) = ctx.find_ref(name) {
                (elem.name().clone(), elem.type_().clone(), elem.anonymous())
            } else {
                panic!("ref not found {}", name);
            }
        },
        ast::ElementType::Complex(ref name, ref complex_type) => {
            let complex_type = flatten_anon_complex(complex_type, ctx, name);
            let type_name = complex_type.name().clone();
            let elem_name = match name {
                None => type_name.clone(),
                Some(ref name) => name.clone()
            };
            ctx.add_content(flat_ast::PacketContent::Complex(complex_type));
            (elem_name, type_name, true)
        }
    };
    let bitset = if let Some(bits) = elem.bits() {
        if let Some(start) = ctx.add_bits(bits) {
            Some(flat_ast::Bitset::new(0, start, format!("bitset{}", ctx.bitsets)))
        } else {
            None
        }
    } else {
        ctx.stop_bits();
        None
    };
    let mut element = flat_ast::Element::new(name, type_, id, init, elem.occurs().clone(),
        elem.size_occurs().clone(), elem.doc().clone(), anonymous, elem.reference(),
        elem.read_write().clone(), elem.bits(), bitset);
    if let Some(ref t) = elem.enum_type() {
        element.set_enum_type(t.clone());
    }
    element
}
