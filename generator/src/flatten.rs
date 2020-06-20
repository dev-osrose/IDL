use ::schema::ast;
use ::flat_ast;
use ::schema::Reader;
use std::fs::File;
use std::collections::HashSet;

struct Context<'a> {
    packet: &'a mut flat_ast::Packet,
    path: Vec<String>,
    complex_types: HashSet<String>,
}

impl<'a> Context<'a> {
    fn add_content(&mut self, content: flat_ast::PacketContent) {
        if let flat_ast::PacketContent::Complex(ref cot) = content {
            self.complex_types.insert(cot.name().clone());
        }
        self.packet.add_content(content);
    }
}

pub fn flatten(search_path: &::std::path::Path, p: &ast::Packet) -> Result<flat_ast::Packet, ::failure::Error> {
    let mut packet = flat_ast::Packet::new(p.doc().clone());
    {
        let mut ctx = Context {
            packet: &mut packet,
            path: Vec::new(),
            complex_types: HashSet::new(),
        };
        flatten_(search_path, p, &mut ctx)?;
    }
    Ok(packet)
}

fn flatten_(search_path: &::std::path::Path, packet: &ast::Packet, ctx: &mut Context) -> Result<(), ::failure::Error> {
    for content in packet.contents() {
        use flat_ast::PacketContent::*;
        match content {
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
    *enum_id += 1;
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
    let cot = flat_ast::ComplexType::new(c.name().clone(), content, c.doc().clone());
    ctx.add_content(flat_ast::PacketContent::Complex(cot));
}

fn flatten_seq(s: &ast::Sequence, ctx: &mut Context) -> flat_ast::Sequence {
    let mut seq = flat_ast::Sequence::new(s.doc().clone());
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
    let mut choice = flat_ast::Choice::new(c.doc().clone());
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
    match c {
        ast::SequenceContent::Element(ref element) => {
            return flatten_element(element, ctx, id);
        },
    };
}

fn flatten_element(elem: &ast::Element, _ctx: &mut Context, id: u32) -> flat_ast::Element {
    let init = elem.init().clone();
    let (name, type_) = match elem.type_() {
        ast::ElementType::Named{ ref name, ref type_ } => (name.clone(), type_.clone()),
    };
    let element = flat_ast::Element::new(name, type_, id, init,
        elem.occurs().clone(), elem.doc().clone());
    element
}
