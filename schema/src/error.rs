use std::convert::From;

#[derive(Fail, Debug)]
pub enum ParseError {
    #[fail(display = "XML read error: {}", _0)]
    XmlError(::xml::reader::Error),
    #[fail(display = "XML read error: {}", _0)]
    Custom(String),
    #[fail(display = "Attribute not found: {}", _0)]
    Attribute(String),
    #[fail(display = "Element not handled: {}", _0)]
    Element(String),
    #[fail(display = "Wrap error: {}", _0)]
    Wrap(::failure::Error),
}

impl From<::xml::reader::Error> for ParseError {
    fn from(e: ::xml::reader::Error) -> Self {
        ParseError::XmlError(e)
    }
}

impl From<::failure::Error> for ParseError {
    fn from(e: ::failure::Error) -> Self {
        ParseError::Wrap(e)
    }
}

impl ParseError {
    pub (crate) fn new<T: Into<String>>(msg: T) -> Self {
        ParseError::Custom(msg.into())
    }
}