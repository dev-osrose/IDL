#[derive(Debug)]
pub struct Packet {
    type_: String,
    contents: Vec<PacketContent>,
    doc: Option<String>,
    class_name: String,
    filename: String
}

#[derive(Debug)]
pub enum PacketContent {
    Include(String, bool),
    Element(Element),
    Simple(SimpleType),
    Complex(ComplexType)
}

#[derive(Debug)]
pub struct ComplexType {
    name: String,
    content: ComplexTypeContent,
    doc: Option<String>,
    anonymous: bool
}

#[derive(Debug)]
pub enum ComplexTypeContent {
    Seq(Sequence),
    Choice(Choice),
    Empty
}

pub use ::packet_schema::ast::Occurs;

#[derive(Debug)]
pub struct Sequence {
    elements: Vec<Element>,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>
}

#[derive(Debug)]
pub struct Choice {
    elements: Vec<Element>,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>
}

#[derive(Debug)]
pub struct Element {
    name: String,
    type_: String,
    id: u32,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    init: ElementInitValue,
    anonymous: bool,
    reference: bool,
    enum_type: Option<String>,
    is_defined: bool
}

#[derive(Debug)]
pub struct SimpleType {
    name: String,
    contents: Vec<SimpleTypeContent>,
    doc: Option<String>
}

#[derive(Debug)]
pub enum SimpleTypeContent {
    Restriction(Restriction)
}

#[derive(Debug)]
pub struct Restriction {
    base: String,
    doc: Option<String>,
    contents: Vec<RestrictionContent>
}

#[derive(Debug)]
pub enum RestrictionContent {
    Enumeration(Enumeration),
    Length(u32),
    MinValue(String),
    MaxValue(String)
}

#[derive(Debug)]
pub struct Enumeration {
    value: String,
    id: i64,
    doc: Option<String>
}

pub use ::packet_schema::ast::ElementInitValue;

impl PacketContent {
    #[inline]
    pub fn type_from_name(content: &PacketContent) -> Option<String> {
        match content {
            PacketContent::Simple(s) => Some(s.name().to_owned()),
            PacketContent::Complex(c) => Some(c.name().to_owned()),
            _ => None
        }
    }

    #[inline]
    pub fn enum_type(content: &PacketContent) -> Option<String> {
        match content {
            PacketContent::Element(elem) => elem.enum_type().to_owned(),
            _ => None
        }
    }

    #[inline]
    pub fn is_type(content: &PacketContent) -> bool {
        match content {
            PacketContent::Simple(_) | PacketContent::Complex(_) => true,
            _ => false
        }
    }
}

impl Packet {
    pub fn new(type_: String
               , doc: Option<String>) -> Self {
        use ::heck::*;
        let name = type_.clone().to_camel_case();
        let (class_name, filename) = if type_.starts_with("Isc") {
            (name.clone(),
             name.clone().to_snake_case())
        } else {
            if type_.starts_with("Pakcs") {
                let name = "Cli".to_string() + &name[5..];
                (name.clone(),
                 name.clone().to_snake_case())
            } else {
                let name = "Srv".to_string() + &name[5..];
                (name.clone(),
                 name.clone().to_snake_case())
            }
        };

        Packet{
            type_,
            contents: Vec::new(),
            doc: doc,
            class_name: class_name,
            filename: filename
        }
    }

    pub fn add_content(&mut self, content: PacketContent) {
        self.contents.push(content);
    }

    pub fn class_name(&self) -> &String {
        &self.class_name
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn type_(&self) -> &String {
        &self.type_
    }

    pub fn contents(&self) -> &[PacketContent] {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut [PacketContent] {
        &mut self.contents
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

impl ComplexType {
    pub fn new(
        name: String,
        content: ComplexTypeContent,
        doc: Option<String>,
        anonymous: bool
    ) -> Self {
        ComplexType{ name, content, doc, anonymous }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn content(&self) -> &ComplexTypeContent {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut ComplexTypeContent {
        &mut self.content
    }

    pub fn anonymous(&self) -> bool {
        self.anonymous
    }
}

impl Sequence {
    pub fn new(
        occurs: Option<Occurs>,
        size_occurs: Option<String>,
        doc: Option<String>
    ) -> Self {
        Sequence{ elements: Vec::new(), occurs, size_occurs, doc }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl Choice {
    pub fn new( occurs: Option<Occurs>, size_occurs: Option<String>
              , doc: Option<String>) -> Self {
        Choice{ elements: Vec::new(), occurs, size_occurs, doc }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl Element {
    pub fn new(name: String, type_: String, id: u32
               , init: ElementInitValue
               , occurs: Option<Occurs>
               , size_occurs: Option<String>
               , doc: Option<String>
               , anonymous: bool
               , reference: bool) -> Self {
        Element{ name, init, type_, id, occurs, size_occurs, doc
                 , anonymous, reference, enum_type: None,
                 is_defined: false }
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn reference(&self) -> bool {
        self.reference
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    
    pub fn type_(&self) -> &String {
        &self.type_
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn init(&self) -> &ElementInitValue {
        &self.init
    }

    pub fn anonymous(&self) -> bool {
        self.anonymous
    }

    pub fn set_enum_type(&mut self, type_: String) {
        self.enum_type = Some(type_);
    }

    pub fn enum_type(&self) -> &Option<String> {
        &self.enum_type
    }

    pub fn set_is_defined(&mut self) {
        self.is_defined = true;
    }

    pub fn is_defined(&self) -> bool {
        self.is_defined
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl SimpleType {
    pub fn new(name: String, doc: Option<String>) -> Self {
        use heck::CamelCase;
        SimpleType{ name: name.to_camel_case(), contents: Vec::new(), doc }
    }

    pub fn add_content(&mut self, content: SimpleTypeContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[SimpleTypeContent] {
        &self.contents
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

impl Restriction {
    pub fn new(base: String, doc: Option<String> ) -> Self {
        Restriction{ base, contents: Vec::new(), doc }
    }

    pub fn add_content(&mut self, content: RestrictionContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[RestrictionContent] {
        &self.contents
    }

    pub fn base(&self) -> &String {
        &self.base
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}


impl Enumeration {
    pub fn new(value: String, id: i64, doc: Option<String>) -> Self {
        Enumeration{ value, id, doc }
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}