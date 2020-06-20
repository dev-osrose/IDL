#[derive(Debug)]
pub struct Packet {
    contents: Vec<PacketContent>,
    doc: Option<String>,
}

#[derive(Debug)]
pub enum PacketContent {
    Element(Element),
    Simple(SimpleType),
    Complex(ComplexType)
}

#[derive(Debug)]
pub struct ComplexType {
    name: String,
    content: ComplexTypeContent,
    doc: Option<String>,
}

#[derive(Debug)]
pub enum ComplexTypeContent {
    Seq(Sequence),
    Choice(Choice),
    Empty
}

pub use ::schema::ast::Occurs;
pub use ::schema::ast::ElementInitValue;

#[derive(Debug, Clone)]
pub struct Sequence {
    elements: Vec<Element>,
    doc: Option<String>,
}

#[derive(Debug)]
pub struct Choice {
    elements: Vec<Element>,
    doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Element {
    name: String,
    type_: String,
    id: u32,
    doc: Option<String>,
    occurs: Option<Occurs>,
    init: ElementInitValue,
    is_defined: bool,
}

#[derive(Debug)]
pub struct SimpleType {
    name: String,
    content: SimpleTypeContent,
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
}

#[derive(Debug)]
pub struct Enumeration {
    value: String,
    id: i64,
    doc: Option<String>
}

impl PacketContent {
    #[inline]
    pub fn type_from_name(&self) -> Option<String> {
        match self {
            PacketContent::Simple(s) => Some(s.name().to_owned()),
            PacketContent::Complex(c) => Some(c.name().to_owned()),
            _ => None
        }
    }

    #[inline]
    pub fn is_type(&self) -> bool {
        match self {
            PacketContent::Simple(_) | PacketContent::Complex(_) => true,
            _ => false
        }
    }
}

impl Packet {
    pub fn new(doc: Option<String>) -> Self {
        Packet{
            contents: Vec::new(),
            doc: doc,
        }
    }

    pub fn add_content(&mut self, content: PacketContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[PacketContent] {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut Vec<PacketContent> {
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
    ) -> Self {
        ComplexType{ name, content, doc, }
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
}

impl Sequence {
    pub fn new(doc: Option<String>) -> Self {
        Sequence{ elements: Vec::new(), doc }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

impl Choice {
    pub fn new(doc: Option<String>) -> Self {
        Choice{ elements: Vec::new(), doc }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    #[allow(dead_code)]
    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

impl Element {
    pub fn new(name: String, type_: String, id: u32
               , init: ElementInitValue
               , occurs: Option<Occurs>
               , doc: Option<String>) -> Self {
        Element{ name, init, type_, id, occurs, doc, is_defined: false }
    }
    
    pub fn name(&self) -> &String {
        &self.name
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
}

impl SimpleType {
    pub fn new(name: String, content: SimpleTypeContent, doc: Option<String>) -> Self {
        use heck::CamelCase;
        SimpleType{ name: name.to_camel_case(), content, doc }
    }

    pub fn content(&self) -> &SimpleTypeContent {
        &self.content
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
