use std;

#[derive(Debug)]
pub struct Packet {
    type_: String,
    contents: Vec<PacketContent>,
    doc: Option<String>
}

#[derive(Debug)]
pub enum PacketContent {
    IncludeXml(String),
    Include(String, bool),
    SimpleType(SimpleType),
    ComplexType(ComplexType),
    Element(Element)
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum RestrictionContent {
    Enumeration(Enumeration),
    Length(u32),
    MinValue(String),
    MaxValue(String)
}

#[derive(Debug, PartialEq, Eq, Ord)]
pub struct Enumeration {
    value: String,
    id: Option<i64>,
    doc: Option<String>
}

impl std::cmp::PartialOrd for Enumeration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

#[derive(Debug)]
pub struct ComplexType {
    name: String,
    content: ComplexTypeContent,
    doc: Option<String>
}

#[derive(Debug)]
pub enum ComplexTypeContent {
    Seq(Sequence),
    Choice(Choice),
    Empty
}

#[derive(Debug, Clone, PartialEq)]
pub enum Occurs {
    Num(String),
    Unbounded
}

#[derive(Debug)]
pub struct Sequence {
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    contents: Vec<SequenceContent>,
    doc: Option<String>
}

#[derive(Debug)]
pub enum SequenceContent {
    Element(Element),
    Choice(Choice),
    Seq(Sequence)
}

#[derive(Debug)]
pub struct Choice {
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    contents: Vec<SequenceContent>,
    doc: Option<String>
}

#[derive(Debug, Clone)]
pub enum ElementInitValue {
    Default(String),
    Create,
    None
}

#[derive(Debug)]
pub struct Element {
    type_: ElementType,
    init: ElementInitValue,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    reference: bool
}

#[derive(Debug)]
pub enum ElementType {
    Named { name: String, type_: String },
    Ref(String),
    Complex(Option<String>, AnonComplexType)
}

#[derive(Debug)]
pub struct AnonComplexType {
    content: ComplexTypeContent,
    doc: Option<String>
}

impl Packet {
    pub fn new(type_: String) -> Self {
        Packet {
            type_: type_,
            contents: Vec::new(),
            doc: None
        }
    }

    pub fn add_content(&mut self, content: PacketContent) {
        self.contents.push(content);
    }

    pub fn type_(&self) -> &String {
        &self.type_
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn contents(&self) -> &[PacketContent] {
        &self.contents
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }
}

impl ComplexType {
    pub fn new(name: String, content: ComplexTypeContent) -> Self {
        ComplexType {
            name: name,
            content: content,
            doc: None
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }

    pub fn content(&self) -> &ComplexTypeContent {
        &self.content
    }
}

impl AnonComplexType {
    pub fn new(content: ComplexTypeContent) -> Self {
        AnonComplexType {
            content: content,
            doc: None
        }
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }

    pub fn content(&self) -> &ComplexTypeContent {
        &self.content
    }
}

impl Sequence {
    pub fn new(occurs: Option<Occurs>, size_occurs: Option<String>, doc: Option<String>) -> Self {
        Sequence {
            contents: Vec::new(),
            occurs: occurs,
            size_occurs: size_occurs,
            doc: doc
        }
    }

    pub fn add_content(&mut self, content: SequenceContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[SequenceContent] {
        &self.contents
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl Choice {
    pub fn new(occurs: Option<Occurs>, size_occurs: Option<String>, doc: Option<String>) -> Self {
        Choice {
            contents: Vec::new(),
            occurs: occurs,
            size_occurs: size_occurs,
            doc: doc
        }
    }

    pub fn add_content(&mut self, content: SequenceContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[SequenceContent] {
        &self.contents
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl Element {
    pub fn new(type_: ElementType, init: ElementInitValue, occurs: Option<Occurs>, size_occurs: Option<String>, reference: bool) -> Self {
        Element {
            type_: type_,
            occurs: occurs,
            size_occurs: size_occurs,
            doc: None,
            init: init,
            reference: reference
        }
    }

    pub fn reference(&self) -> bool {
        self.reference
    }

    pub fn type_(&self) -> &ElementType {
        &self.type_
    }

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }

    pub fn init(&self) -> &ElementInitValue {
        &self.init
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }
}

impl SimpleType {
    pub fn new(name: String) -> Self {
        SimpleType {
            name: name,
            contents: Vec::new(),
            doc: None
        }
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

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }
}

impl Restriction {
    pub fn new(base: String) -> Self {
        Restriction {
            base: base,
            contents: Vec::new(),
            doc: None
        }
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

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }
}

impl Enumeration {
    pub fn new(value: String, id: Option<i64>, doc: Option<String>) -> Self {
        Enumeration {
            value: value,
            id: id,
            doc: doc
        }
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn id(&self) -> &Option<i64> {
        &self.id
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}