use ::ast::*;
use ::error::ParseError;
use ::attributes::Attributes;
use ::reader::Reader;
use std::fmt::Debug;

type Result<T> = ::std::result::Result<T, ParseError>;

pub fn parse_packet(r: &mut Reader, attrs: Attributes) -> Result<Packet> {
    packet(r, attrs)
}

pub fn parse_simple_type(r: &mut Reader, attrs: Attributes) -> Result<Packet> {
    use self::PacketContent::*;
    let mut packet = Packet::new("tmp".to_string());
    packet.add_content(SimpleType(simple_type(r, attrs)?));
    Ok(packet)
}

pub fn parse_complex_type(r: &mut Reader, attrs: Attributes) -> Result<Packet> {
    use self::PacketContent::*;
    let mut packet = Packet::new("tmp".to_string());
    packet.add_content(ComplexType(complex_type(r, attrs)?));
    Ok(packet)
}

#[derive(Debug)]
enum Either<A: Debug, B: Debug> {
    A(A),
    B(B)
}

fn packet(r: &mut Reader, attrs: Attributes) -> Result<Packet> {
    let type_ = attrs.get("ePacketType")?;
    let mut packet = Packet::new(type_);

    use self::PacketContent::*;
    use self::Either::*;
    for item in r.map(&[
        ("includeXml", &|r, attrs| Ok(A(include_xml(r, attrs)?))),
        ("include", &|r, attrs| Ok(A(include(r, attrs)?))),
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
    let occurs = attrs.parse_opt("occurs")?;
    let size_occurs = attrs.parse_opt("occursSize")?;
    let (doc, contents) = seq_or_choice_children(r, attrs)?;
    let mut seq = Sequence::new(occurs, size_occurs, doc);
    for content in contents {
        seq.add_content(content);
    }
    Ok(seq)
}

fn choice(r: &mut Reader, attrs: Attributes) -> Result<Choice> {
    let occurs = attrs.parse_opt("occurs")?;
    let size_occurs = attrs.parse_opt("occursSize")?;
    let (doc, contents) = seq_or_choice_children(r, attrs)?;
    let mut choice = Choice::new(occurs, size_occurs, doc);
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
    for content in r.map(&[
        ("element", &|r, attrs| Ok(A(Element(element(r, attrs)?)))),
        ("choice", &|r, attrs| Ok(A(Choice(choice(r, attrs)?)))),
        ("sequence", &|r, attrs| Ok(A(Seq(seq(r, attrs)?)))),
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
    let name = attrs.get("name")?;
    let (content, doc) = complex_content(r)?;
    let mut cot = ComplexType::new(name, content);
    if let Some(doc) = doc {
        cot.set_doc(doc);
    }
    Ok(cot)
}

fn include(_: &mut Reader, attrs: Attributes) -> Result<PacketContent> {
    let path = attrs.get("path")?;
    let system = attrs.get_or("system", false);
    Ok(PacketContent::Include(path, system))
}

fn include_xml(_: &mut Reader, attrs: Attributes) -> Result<PacketContent> {
    let path = attrs.get("path")?;
    Ok(PacketContent::IncludeXml(path))
}

fn simple_type(r: &mut Reader, attrs: Attributes) -> Result<SimpleType> {
    let name = attrs.get("name")?;
    let mut sit = SimpleType::new(name);

    use self::Either::*;
    for item in r.map(&[
        ("restriction", &|r, attrs| Ok(A(restriction(r, attrs)?))),
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?)))
    ])? {
        match item {
            A(item) => sit.add_content(SimpleTypeContent::Restriction(item)),
            B(doc) => sit.set_doc(doc)
        };
    }
    Ok(sit)
}

fn restriction(r: &mut Reader, attrs: Attributes) -> Result<Restriction> {
    let base = attrs.get::<String>("base")?;
    let mut restrict = Restriction::new(base);

    use self::Either::*;
    use self::RestrictionContent::*;
    for content in r.map(&[
        ("enumeration", &|r, attrs| Ok(A(Enumeration(enumeration(r, attrs)?)))),
        ("minValue", &|_, attrs| Ok(A(MinValue(attrs.get("value")?)))),
        ("maxValue", &|_, attrs| Ok(A(MaxValue(attrs.get("value")?)))),
        ("length", &|_, attrs| Ok(A(Length(attrs.get("value")?)))),
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
    let value = attrs.get("value")?;
    let id = attrs.parse_opt("id")?;
    let mut doc = None;
    for documentation in r.map(&[("documentation", &documentation)])? {
        doc = Some(documentation);
    }

    Ok(Enumeration::new(value, id, doc))
}

fn element(r: &mut Reader, attrs: Attributes) -> Result<Element> {
    let type_ = attrs.get_opt("type");
    let name = attrs.get_opt("name");
    let default = attrs.get_opt("default");
    let occurs = attrs.parse_opt("occurs")?;
    let size_occurs = attrs.parse_opt("occursSize")?;
    let reference = attrs.get_or("ref", false);
    let read_write = attrs.get_opt("readWrite");
    let enum_type = attrs.get_opt("enum");
    let bits = attrs.parse_opt("bits")?;
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
    let mut type_ = match (type_, name.clone()) {
        (Some(type_), Some(name)) => Some(ElementType::Named{name, type_}),
        _ => None
    };

    use self::Either::*;
    for item in r.map(&[
        ("documentation", &|r, attrs| Ok(B(documentation(r, attrs)?))),
        ("complexType", &|r, attrs| Ok(A(anon_complex_type(r, attrs)?)))
    ])? {
            match item {
                A(item) => type_ = Some(ElementType::Complex(name.clone(), item)),
                B(doc_) => doc = Some(doc_)
            }
    }
    type_.map(|type_| {
        let mut elem = Element::new(type_, init, occurs, size_occurs, reference, read_write, enum_type, bits);
        if let Some(doc) = doc {
            elem.set_doc(doc);
        }
        elem
    }).ok_or_else(|| ParseError::new("name and/or type not found for element"))
}

fn documentation(r: &mut Reader, _: Attributes) -> Result<String> {
    Ok(r.read_text()?.trim().to_string())
}

fn anon_complex_type (r: &mut Reader, _: Attributes) -> Result<AnonComplexType> {
    let (content, doc) = complex_content(r)?;
    let mut cot = AnonComplexType::new(content);
    if let Some(doc) = doc {
        cot.set_doc(doc);
    }
    Ok(cot)
}
