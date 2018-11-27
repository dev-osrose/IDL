use ::packet_schema::ast;
use ::flat_ast;
use ::packet_schema::Reader;
use std::fs::File;
use std::collections::HashSet;

struct Context<'a> {
    packet: &'a mut flat_ast::Packet,
    path: Vec<String>,
    complex_types: HashSet<String>
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
}

pub fn flatten(search_path: &str, p: &ast::Packet) -> Result<flat_ast::Packet, ::failure::Error> {
    let mut packet = flat_ast::Packet::new(p.type_().clone(), p.doc().clone());
    {
        let mut ctx = Context {
            packet: &mut packet,
            path: Vec::new(),
            complex_types: HashSet::new()
        };
        flatten_(search_path, p, &mut ctx)?;
    }
    Ok(packet)
}

fn flatten_(search_path: &str, packet: &ast::Packet, ctx: &mut Context) -> Result<(), ::failure::Error> {
    for content in packet.contents() {
        use flat_ast::PacketContent::*;
        match content {
            ast::PacketContent::Include(ref path, system) => {
                ctx.add_content(Include(path.clone(), *system));
            },
            ast::PacketContent::IncludeXml(ref location) => {
                let filenm = format!("{}/{}", search_path, location);
                println!("Including {}", filenm);
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
                let complex = flatten_complex(complex, ctx);
            },
            ast::PacketContent::Element(ref element) => {
                let element = flatten_element(element, ctx, 0);
                ctx.add_content(Element(element));
            }
        }
    }
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
    let content = match c.content() {
        ComplexTypeContent::Choice(ref c) => Choice(flatten_choice(c, ctx)),
        ComplexTypeContent::Seq(ref s) => Seq(flatten_seq(s, ctx)),
        ComplexTypeContent::Empty => Empty
    };
    let cot = flat_ast::ComplexType::new(c.name().clone(), content, c.doc().clone(), false);
    ctx.add_content(flat_ast::PacketContent::Complex(cot));
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
    let content = match c.content() {
        ComplexTypeContent::Choice(ref c) => Choice(flatten_choice(c, ctx)),
        ComplexTypeContent::Seq(ref s) => Seq(flatten_seq(s, ctx)),
        ComplexTypeContent::Empty => Empty
    };
    let name = ctx.get_anon_name();
    ctx.path.pop();
    flat_ast::ComplexType::new(name, content, c.doc().clone(), true)
}

fn flatten_seq(s: &ast::Sequence, ctx: &mut Context) -> flat_ast::Sequence {
    let mut seq = flat_ast::Sequence::new(s.occurs().clone(), s.doc().clone());
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
    let mut choice = flat_ast::Choice::new(c.occurs().clone(), c.doc().clone());
    let mut max_id = 0;
    for content in c.contents() {
        let element = flatten_seq_content(content, ctx, max_id);
        if max_id <= element.id() {
            max_id = element.id() + 1;
        }
        choice.add_element(element);
    }
    choice
}

fn flatten_seq_content(c: &ast::SequenceContent, ctx: &mut Context, id: u32) -> flat_ast::Element {
    let (name, occurs, doc, content) = match c {
        ast::SequenceContent::Element(ref element) => {
            return flatten_element(element, ctx, id);
        },
        ast::SequenceContent::Choice(ref choice) => {
            ctx.path.push("Choice".to_string());
            let choice = flatten_choice(choice, ctx);
            let doc = choice.doc().clone();
            let occurs = choice.occurs().clone();
            let name = ctx.get_anon_name();
            ctx.path.pop();
            let content = flat_ast::ComplexTypeContent::Choice(choice);
            (name, occurs, doc, content)
        },
        ast::SequenceContent::Seq(ref seq) => {
            ctx.path.push("Sequence".to_string());
            let seq = flatten_seq(seq, ctx);
            let occurs = seq.occurs().clone();
            let doc = seq.doc().clone();
            let name = ctx.get_anon_name();
            ctx.path.pop();
            let content = flat_ast::ComplexTypeContent::Seq(seq);
            (name, occurs, doc, content)
        }
    };

    let complex = flat_ast::ComplexType::new(name.clone(), content, doc.clone(), true);
    ctx.add_content(flat_ast::PacketContent::Complex(complex));
    flat_ast::Element::new(name.clone(), name.clone(), id, flat_ast::ElementInitValue::None, occurs, doc, true, true)
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
    flat_ast::Element::new(name, type_, id, init, elem.occurs().clone(), elem.doc().clone(), anonymous, elem.reference())
}