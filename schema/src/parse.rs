use ::ast::*;
use ::error::ParseError;
use ::attributes::Attributes;
use ::reader::Reader;
use std::fmt::Debug;

type Result<T> = ::std::result::Result<T, ParseError>;

pub fn parse_packet(r: &mut Reader, attrs: Attributes) -> Result<Either<Packet, PacketContent>> {
    trace!("reading packet in root");
    Ok(Either::A(packet(r, attrs)?))
}

pub fn parse_simple_type(r: &mut Reader, attrs: Attributes) -> Result<Either<Packet, PacketContent>> {
    use self::PacketContent::*;
    trace!("reading simpleType in root");
    Ok(Either::B(SimpleType(simple_type(r, attrs)?)))
}

pub fn parse_complex_type(r: &mut Reader, attrs: Attributes) -> Result<Either<Packet, PacketContent>> {
    use self::PacketContent::*;
    trace!("reading complexType in root");
    Ok(Either::B(ComplexType(complex_type(r, attrs)?)))
}

pub fn parse_include_xml(r: &mut Reader, attrs: Attributes) -> Result<Either<Packet, PacketContent>> {
    trace!("reading includeXml in root");
    Ok(Either::B(include_xml(r, attrs)?))
}

#[derive(Debug)]
pub enum Either<A: Debug, B: Debug> {
    A(A),
    B(B)
}

fn packet(r: &mut Reader, _attrs: Attributes) -> Result<Packet> {
    trace!("reading packet");
    let mut packet = Packet::new();

    use self::PacketContent::*;
    use self::Either::*;
    for item in r.map(&[
        ("includeXml", &|r, attrs| Ok(A(include_xml(r, attrs)?))),
        ("element", &|r, attrs| Ok(A(Element(element(r, attrs)?)))),
        ("simpleType", &|r, attrs| Ok(A(SimpleType(simple_type(r, attrs)?)))),
        ("complexType", &|r, attrs| Ok(A(ComplexType(complex_type(r, attrs)?)))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match item {
            A(item) => packet.add_content(item),
            B(doc) => packet.set_doc(doc)
        }
    }
    Ok(packet)
}

fn complex_content(r: &mut Reader) -> Result<(ComplexTypeContent, Option<String>)> {
    use self::Either::*;
    use self::ComplexTypeContent::*;

    let mut doc = None;
    let mut content = ComplexTypeContent::Empty;
    trace!("reading complex_content");
    for item in r.map(&[
        ("sequence", &|r, attrs| Ok(A(Seq(seq(r, attrs)?)))),
        ("choice", &|r, attrs| Ok(A(Choice(choice(r, attrs)?)))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match item {
            A(item) => content = item,
            B(doc_) => doc = Some(doc_)
        };
    }

    Ok((content, doc))
}

fn seq(r: &mut Reader, attrs: Attributes) -> Result<Sequence> {
    trace!("reading sequence");
    let (doc, contents) = seq_or_choice_children(r, attrs)?;
    let mut seq = Sequence::new(doc);
    for content in contents {
        seq.add_content(content);
    }
    Ok(seq)
}

fn choice(r: &mut Reader, attrs: Attributes) -> Result<Choice> {
    trace!("reading choice");
    let (doc, contents) = seq_or_choice_children(r, attrs)?;
    let mut choice = Choice::new(doc);
    for content in contents {
        choice.add_content(content);
    }
    Ok(choice)
}

fn seq_or_choice_children(r: &mut Reader, _: Attributes) -> Result<(Option<String>, Vec<SequenceContent>)> {
    let mut children = Vec::new();
    let mut doc = None;

    use self::Either::*;
    use self::SequenceContent::*;
    trace!("reading sequence/choice content");
    for content in r.map(&[
        ("element", &|r, attrs| Ok(A(Element(element(r, attrs)?)))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match content {
            A(content) => children.push(content),
            B(doc_) => doc = Some(doc_)
        };
    }
    Ok((doc, children))
}

fn complex_type(r: &mut Reader, attrs: Attributes) -> Result<ComplexType> {
    trace!("reading complex_type");
    let name = attrs.get("name")?;
    let (content, doc) = complex_content(r)?;
    let mut cot = ComplexType::new(name, content);
    if let Some(doc) = doc {
        cot.set_doc(doc);
    }
    Ok(cot)
}

fn include_xml(_: &mut Reader, attrs: Attributes) -> Result<PacketContent> {
    trace!("reading include_xml");
    let path = attrs.get("path")?;
    Ok(PacketContent::IncludeXml(path))
}

fn simple_type(r: &mut Reader, attrs: Attributes) -> Result<SimpleType> {
    trace!("reading simple_type");
    let name = attrs.get("name")?;
    let mut res = None;
    let mut doc = None;

    use self::Either::*;
    for item in r.map(&[
        ("restriction", &|r, attrs| Ok(A(restriction(r, attrs)?))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match item {
            A(item) => res = Some(SimpleTypeContent::Restriction(item)),
            B(doc_) => doc = Some(doc_)
        };
    }
    if let Some(res) = res {
        Ok(SimpleType::new(name, res, doc))
    } else {
        Err(ParseError::new(format!("No restriction found in simple type {}", name)))
    }
}

fn restriction(r: &mut Reader, attrs: Attributes) -> Result<Restriction> {
    trace!("reading restriction");
    let base = attrs.get::<String>("base")?;
    let mut restrict = Restriction::new(base);

    use self::Either::*;
    use self::RestrictionContent::*;
    for content in r.map(&[
        ("enumeration", &|r, attrs| Ok(A(Enumeration(enumeration(r, attrs)?)))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match content {
            A(content) => restrict.add_content(content),
            B(doc) => restrict.set_doc(doc)
        }
    }

    Ok(restrict)
}

fn enumeration(r: &mut Reader, attrs: Attributes) -> Result<Enumeration> {
    trace!("reading enumeration");
    let value = attrs.get("value")?;
    let id = attrs.parse_opt("id")?;
    let mut doc = None;
    for documentation in r.map(&[("documentation", &documentation)])? {
        doc = Some(documentation);
    }

    Ok(Enumeration::new(value, id, doc))
}

fn element(r: &mut Reader, attrs: Attributes) -> Result<Element> {
    trace!("reading element");
    let type_ = attrs.get_opt("type");
    let name = attrs.get_opt("name");
    let default = attrs.get_opt("default");
    let occurs = attrs.parse_opt("occurs")?;
    let mut doc = None;
    let init = match default {
        Some(def) => {
            if def == "create" {
                ElementInitValue::Create
            } else if def == "none" {
                ElementInitValue::None
            } else {
                ElementInitValue::Default(def)
            }
        },
        None => ElementInitValue::Create
    };
    let type_ = match (type_, name.clone()) {
        (Some(type_), Some(name)) => Some(ElementType::Named{name, type_}),
        _ => None
    };

    for item in r.map(&[
        ("documentation", &|r, attrs| Ok(documentation(r, attrs)?)),
    ])? {
        doc = Some(item.clone());
    }
    type_.map(|type_| {
        let mut elem = Element::new(type_, init, occurs);
        if let Some(doc) = doc {
            elem.set_doc(doc);
        }
        elem
    }).ok_or_else(|| ParseError::new("name and/or type not found for element"))
}

fn documentation(r: &mut Reader, _: Attributes) -> Result<String> {
    trace!("reading documentation");
    Ok(r.read_text()?.trim().to_string())
}