use ::error::ParseError;
use ::xml::{attribute::OwnedAttribute};
use std::collections::HashMap;
type Result<T> = ::std::result::Result<T, ParseError>;

pub struct Attributes {
    attrs: HashMap<String, String>
}

pub trait Parse: Sized {
    fn parse(&str) -> Result<Self>;
}

impl Attributes {
    pub fn new(attr_vec: &Vec<OwnedAttribute>) -> Self {
        let mut attr_col = Attributes { attrs: HashMap::new() };
        for attr in attr_vec {
            attr_col.attrs.insert(attr.name.local_name.clone(), attr.value.clone());
        }
        attr_col
    }

    pub fn get<P: Parse>(&self, name: &str) -> Result<P> {
        let val = self.attrs.get(&name.to_string()).ok_or(ParseError::Attribute(name.into()));
        match val {
            Err(e) => Err(e),
            Ok(val) => P::parse(&val)
        }
    }

    pub fn parse_opt<P: Parse>(&self, name: &str) -> Result<Option<P>> {
        let res = match self.attrs.get(&name.to_string()) {
            Some(val) => Some(P::parse(val)?),
            None => None
        };
        Ok(res)
    }

    pub fn get_opt(&self, name: &str) -> Option<String> {
        self.attrs.get(&name.to_string()).map(|val| val.clone())
    }

    pub fn get_or<P: Parse>(&self, name: &str, default: P) -> P {
        match self.get::<P>(name) {
            Ok(val) => val,
            _ => default
        }
    }
}

impl Parse for bool {
    fn parse(val: &str) -> Result<Self> {
        match val {
            | "true" => Ok(true),
            | "false" => Ok(false),
            | _ => Err(ParseError::new(format!("Invalid bool {}", val)))
        }
    }
}

impl Parse for i8 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match i8::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid i8 {} {:?}", val, e)))
        }
    }
}

impl Parse for u8 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match u8::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid u8 {} {:?}", val, e)))
        }
    }
}

impl Parse for i16 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match i16::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid i16 {} {:?}", val, e)))
        }
    }
}

impl Parse for u16 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match u16::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid u16 {} {:?}", val, e)))
        }
    }
}

impl Parse for i32 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match i32::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid i32 {} {:?}", val, e)))
        }
    }
}

impl Parse for u32 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match u32::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid u32 {} {:?}", val, e)))
        }
    }
}

impl Parse for i64 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match i64::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid i64 {} {:?}", val, e)))
        }
    }
}

impl Parse for u64 {
    fn parse(val: &str) -> Result<Self> {
        let (idx, radix) = if val.starts_with("0x") {
            (2, 16)
        } else {
            (0, 10)
        };
        let val = &val[idx..];
        match u64::from_str_radix(val, radix) {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid u64 {} {:?}", val, e)))
        }
    }
}

impl Parse for f32 {
    fn parse(val: &str) -> Result<Self> {
        match val.parse::<f32>() {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid f32 {} {:?}", val, e)))
        }
    }
}

impl Parse for f64 {
    fn parse(val: &str) -> Result<Self> {
        match val.parse::<f64>() {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::new(format!("Invalid f64 {} {:?}", val, e)))
        }
    }
}

impl Parse for ::ast::Occurs {
    fn parse(val: &str) -> Result<Self> {
        use ::ast::Occurs;
        match val {
            "unbounded" => Ok(Occurs::Unbounded),
            _ => match u32::parse(val) {
                Ok(n) => Ok(Occurs::Num(n)),
                Err(_) => Err(ParseError::new(format!("{} is not a num", val)))
            }
        }
    }
}

impl Parse for String {
    fn parse(val: &str) -> Result<Self> {
        Ok(val.to_string())
    }
}