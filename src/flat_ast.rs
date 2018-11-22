#[derive(Debug)]
pub struct Packet {
    type_: String,
    contents: Vec<PacketContent>,
    doc: Option<String>
}

#[derive(Debug)]
pub enum PacketContent {
    Include(String),
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
    occurs: Option<Occurs>
}

#[derive(Debug)]
pub struct Choice {
    elements: Vec<Element>,
    doc: Option<String>,
    occurs: Option<Occurs>
}

#[derive(Debug)]
pub struct Element {
    name: String,
    type_: String,
    id: u32,
    doc: Option<String>,
    occurs: Option<Occurs>,
    init: ElementInitValue,
    anonymous: bool
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

impl Packet {
    pub fn new(type_: String
               , doc: Option<String>) -> Self {
        Packet{
            type_,
            contents: Vec::new(),
            doc: doc
        }
    }

    pub fn add_content(&mut self, content: PacketContent) {
        self.contents.push(content);
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
        doc: Option<String>
    ) -> Self {
        Sequence{ elements: Vec::new(), occurs, doc }
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
}

impl Choice {
    pub fn new( occurs: Option<Occurs>
              , doc: Option<String>) -> Self {
        Choice{ elements: Vec::new(), occurs, doc }
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
}

impl Element {
    pub fn new(name: String, type_: String, id: u32
               , init: ElementInitValue
               , occurs: Option<Occurs>
               , doc: Option<String>
               , anonymous: bool) -> Self {
        Element{ name, init, type_, id, occurs, doc
                 , anonymous }
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

    pub fn anonymous(&self) -> bool {
        self.anonymous
    }
}

impl SimpleType {
    pub fn new(name: String, doc: Option<String>) -> Self {
        SimpleType{ name, contents: Vec::new(), doc }
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