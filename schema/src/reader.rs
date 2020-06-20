use std::io::Read;
use std::fmt::Debug;
use ::ast::Packet;
use ::error::ParseError;
use ::attributes::Attributes;
use ::xml::reader::{EventReader,XmlEvent};
use ::parse::Either;

pub struct Reader {
    reader: EventReader<Box<dyn Read>>,
    path: Vec<String>,
    event:   Option<XmlEvent>,
}

impl Reader {
    pub fn load_packet<R: Read + 'static>(r: R) -> Result<Packet, ::failure::Error> {
        let mut reader = Reader::new(Box::new(r));
        reader.read()
    }
    
    pub fn new(source: Box<dyn Read>) -> Self {
        let reader = EventReader::new(source);
        Reader{ reader, path: Vec::new(), event: None }
    }

    fn next(&mut self) -> Result<XmlEvent, ::xml::reader::Error> {
        match self.event.take() {
            Some(event) => Ok(event),
            _ => Ok(self.reader.next()?),
        }
    }

    fn read(&mut self) -> Result<Packet, ::failure::Error> {
        if let XmlEvent::StartDocument{..} = self.next()? {
            let mut tmp_packet = Packet::new();
            for item in self.map(&[
                                ("packet", &::parse::parse_packet),
                                ("simpleType", &::parse::parse_simple_type),
                                ("complexType", &::parse::parse_complex_type),
                                ("includeXml", &::parse::parse_include_xml)
            ])? {
                match item {
                    Either::A(mut packet) => {
                        for content in tmp_packet.into_contents() {
                            packet.add_content(content);
                        }
                        return Ok(packet);
                    },
                    Either::B(content) => tmp_packet.add_content(content)
                }
            }
            return Ok(tmp_packet);
        }
        Err(ParseError::new("Expecting startDocument").into())
    }

    fn read_node<Out: Debug>(
        &mut self,
        alts: &[(&'static str, & dyn Fn( &mut Reader, Attributes) -> Result<Out, ParseError> )]
    ) -> Result<Option<Out>, ParseError>
    {
        let mut result = None;
        use self::XmlEvent::*;
        loop {
            let event = self.next()?;
            match event {
                Whitespace(_) => continue,
                StartElement{name, attributes, ..} => {
                    let local_nm = name.local_name;
                    if result.is_some() {
                        //We are reading a child node most likely means the
                        //  parent node parsing function didn't explicitly
                        //  handle this node
                        let msg = format!("Parse error saw {}. Does {} reader explicitly handle this",
                                          local_nm, self.path());
                        return Err(ParseError::new(msg))
                    }
                    self.path.push(local_nm.clone());

                    for (nm, func) in alts {
                        if nm == &local_nm.as_str() {
                            let attributes = Attributes::new(&attributes);
                            result = Some(func(self, attributes)?);
                        }
                    }
                    if result.is_none() {
                        return Err(ParseError::Element(self.path()));
                    }
                },
                EndElement{..} => {
                    self.path.pop();
                    return Ok(result)
                },
                event => {
                    let msg = format!("Unhandled event {:?}", event);
                    return Err(ParseError::new(msg))
                }
            }
        }
    }

    fn path(&self) -> String {
        let mut path = String::new();
        for elem in &self.path {
            path = path + "::" + &elem;
        }
        path
    }

    //Read the child nodes where there is exactly one and it is required.
    pub (crate) fn required<Out: Debug>(
        &mut self,
        alts: &[(&'static str, &dyn Fn( &mut Reader, Attributes) -> Result<Out, ParseError>) ]
    ) -> Result<Out, ParseError>
    {
        match self.read_node(alts)? {
            Some(result) => Ok(result),
            None => {
                let mut msg = "Expected one of ".to_string();
                for (name, _) in alts {
                    msg = msg + ", " + name;
                }
                Err(ParseError::new(msg))
            }
        }
    }

    //Read the child nodes there there are either optional, reapeated or
    //    required
    pub (crate) fn map<Out: Debug>(
        &mut self,
        alts: &[(&'static str, & dyn Fn( &mut Reader, Attributes) -> Result<Out, ParseError> )]
    ) -> Result<Vec<Out>, ParseError>
    {
        let mut results = Vec::new();

        loop {
            let event = self.next()?;
            match event {
                XmlEvent::Whitespace(_)  => continue,
                XmlEvent::EndElement{..} => {
                    //Store the event so our caller will get it too
                    self.event = Some(event);
                    break
                },
                XmlEvent::EndDocument{..} => {
                    self.event = Some(event);
                    break
                },
                _ => {
                    //Store the event so it can be retrieved by self.required
                    self.event = Some(event);
                    let result = self.required(alts)?;
                    results.push(result);
                }
            }
        }
        Ok(results)
    }


    pub (crate) fn read_text(&mut self) -> Result<String, ParseError>
    {
        use self::XmlEvent::*;
        loop {
            match self.next()? {
                Whitespace(_)   => return Ok("".to_string()),
                Characters(val) => return Ok(val),
                event => {
                    let msg = format!("Unhandled event {:?}", event);
                    return Err(ParseError::new(msg))
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn create_cursor(s: &'static str) -> std::io::Cursor<&str> {
        std::io::Cursor::new(s)
    }

    #[test]
    fn empty_packet() {
        let c = create_cursor("<packet></packet>");
        let packet = Reader::load_packet(c);
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(0, packet.contents().len());
    }

    #[test]
    fn empty_packet_error() {
        let c = create_cursor("<packet>");
        let packet = Reader::load_packet(c);
        assert_eq!(true, packet.is_err());
        assert_eq!("XML read error: 1:9 Unexpected end of stream: still inside the root element", packet.err().unwrap().to_string());
    }

    #[test]
    fn empty_packet_doc() {
        let packet = Reader::load_packet(create_cursor("<packet><documentation>plop</documentation></packet>"));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(true, packet.doc().is_some());
        assert_eq!("plop", packet.doc().as_ref().unwrap());
    }

    #[test]
    fn packet_one_empty_element() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element />
        </packet>
        "#));
        assert_eq!(false, packet.is_ok());
        assert_eq!("XML read error: name and/or type not found for element", packet.err().unwrap().to_string());
    }

    #[test]
    fn packet_one_element() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element name = "test" type = "u8" />
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Create, None);
        assert_eq!(PacketContent::Element(elem), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_one_element_init() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element name = "test" type = "u8" default = "42"/>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Default("42".to_string()), None);
        assert_eq!(PacketContent::Element(elem), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_one_element_init_none() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element name = "test" type = "u8" default = "none"/>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::None, None);
        assert_eq!(PacketContent::Element(elem), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_one_element_occurs() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element name = "test" type = "u8" occurs = "42"/>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Create, Some(Occurs::Num(42)));
        assert_eq!(PacketContent::Element(elem), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_one_element_occurs_unbounded() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <element name = "test" type = "u8" occurs = "unbounded"/>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Create, Some(Occurs::Unbounded));
        assert_eq!(PacketContent::Element(elem), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_simple_type() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <simpleType name = "Test">
                <restriction base = "string">
                    <enumeration value = "ZERO" />
                    <enumeration value = "TWO" id = "2" />
                </restriction>
            </simpleType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let mut s = SimpleType::new("Test".to_string());
        let mut r = Restriction::new("string".to_string());
        r.add_content(RestrictionContent::Enumeration(Enumeration::new("ZERO".to_string(), None, None)));
        r.add_content(RestrictionContent::Enumeration(Enumeration::new("TWO".to_string(), Some(2), None)));
        s.add_content(SimpleTypeContent::Restriction(r));
        assert_eq!(PacketContent::SimpleType(s), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_simple_type_elem() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <simpleType name = "Test">
                <restriction base = "string">
                    <enumeration value = "ZERO" />
                    <enumeration value = "TWO" id = "2" />
                </restriction>
            </simpleType>
            <element name = "test" type = "Test" />
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(2, packet.contents().len());
        let mut s = SimpleType::new("Test".to_string());
        let mut r = Restriction::new("string".to_string());
        r.add_content(RestrictionContent::Enumeration(Enumeration::new("ZERO".to_string(), None, None)));
        r.add_content(RestrictionContent::Enumeration(Enumeration::new("TWO".to_string(), Some(2), None)));
        s.add_content(SimpleTypeContent::Restriction(r));
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "Test".to_string()}, ElementInitValue::Create, None);
        assert_eq!(PacketContent::SimpleType(s), packet.contents()[0]);
        assert_eq!(PacketContent::Element(elem), packet.contents()[1]);
    }

    #[test]
    fn packet_complex_type_empty() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <complexType name = "Test">
            </complexType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let s = ComplexType::new("Test".to_string(), ComplexTypeContent::Empty);
        assert_eq!(PacketContent::ComplexType(s), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_complex_type_sequence() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <complexType name = "Test">
                <sequence>
                </sequence>
            </complexType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let s = Sequence::new(None);
        let s = ComplexType::new("Test".to_string(), ComplexTypeContent::Seq(s));
        assert_eq!(PacketContent::ComplexType(s), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_complex_type_sequence_one_elem() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <complexType name = "Test">
                <sequence>
                    <element name = "test" type = "u8" />
                </sequence>
            </complexType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let mut s = Sequence::new(None);
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Create, None);
        s.add_content(SequenceContent::Element(elem));
        let s = ComplexType::new("Test".to_string(), ComplexTypeContent::Seq(s));
        assert_eq!(PacketContent::ComplexType(s), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_complex_type_choice() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <complexType name = "Test">
                <choice>
                </choice>
            </complexType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let c = Choice::new(None);
        let s = ComplexType::new("Test".to_string(), ComplexTypeContent::Choice(c));
        assert_eq!(PacketContent::ComplexType(s), *packet.contents().last().unwrap());
    }

    #[test]
    fn packet_complex_type_choice_one_elem() {
        let packet = Reader::load_packet(create_cursor(r#"
        <packet>
            <complexType name = "Test">
                <choice>
                    <element name = "test" type = "u8" />
                </choice>
            </complexType>
        </packet>
        "#));
        assert_eq!(true, packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(1, packet.contents().len());
        let mut c = Choice::new(None);
        let elem = Element::new(ElementType::Named{name: "test".to_string(), type_: "u8".to_string()}, ElementInitValue::Create, None);
        c.add_content(SequenceContent::Element(elem));
        let s = ComplexType::new("Test".to_string(), ComplexTypeContent::Choice(c));
        assert_eq!(PacketContent::ComplexType(s), *packet.contents().last().unwrap());
    }
}