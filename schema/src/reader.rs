use std::io::Read;
use std::fmt::Debug;
use ::ast::Packet;
use ::error::ParseError;
use ::attributes::Attributes;
use ::xml::reader::{EventReader,XmlEvent};

pub struct Reader {
    reader: EventReader<Box<dyn Read>>,
    path: Vec<String>,
    event:   Option<XmlEvent>,
}

impl Reader {
    pub fn load_packet<R: Read+ 'static>(r: R) -> Result<Packet, ::failure::Error> {
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
            match self.required(&[ ("packet", &::parse::parse_packet) ]) {
                Ok(packet) => return Ok(packet),
                Err(err)   => {
                    use super::xml::common::Position;
                    use ::failure::Fail;
                    let msg = format!("at {} : {}", self.reader.position(), err);
                    return Err(err.context(msg).into())
                }
            }
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
                }
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