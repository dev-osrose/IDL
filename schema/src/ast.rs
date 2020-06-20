use std;

#[derive(Debug)]
pub struct Packet {
    contents: Vec<PacketContent>,
    doc: Option<String>
}

#[derive(Debug, PartialEq)]
pub enum PacketContent {
    IncludeXml(String),
    SimpleType(SimpleType),
    ComplexType(ComplexType),
    Element(Element)
}

#[derive(Debug, PartialEq)]
pub struct SimpleType {
    name: String,
    content: SimpleTypeContent,
    doc: Option<String>
}

#[derive(Debug, PartialEq)]
pub enum SimpleTypeContent {
    Restriction(Restriction)
}

#[derive(Debug, PartialEq)]
pub struct Restriction {
    base: String,
    doc: Option<String>,
    contents: Vec<RestrictionContent>
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum RestrictionContent {
    Enumeration(Enumeration),
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

#[derive(Debug, PartialEq)]
pub struct ComplexType {
    name: String,
    content: ComplexTypeContent,
    doc: Option<String>
}

#[derive(Debug, PartialEq)]
pub enum ComplexTypeContent {
    Seq(Sequence),
    Choice(Choice),
    Empty
}

#[derive(Debug, Clone, PartialEq)]
pub enum Occurs {
    Num(u16),
    Unbounded
}

#[derive(Debug, PartialEq)]
pub struct Sequence {
    contents: Vec<SequenceContent>,
    doc: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum SequenceContent {
    Element(Element),
}

#[derive(Debug, PartialEq)]
pub struct Choice {
    contents: Vec<SequenceContent>,
    doc: Option<String>
}

#[derive(Debug, PartialEq, Clone)]
pub enum ElementInitValue {
    Default(String),
    Create,
    None
}

#[derive(Debug, PartialEq)]
pub struct Element {
    type_: ElementType,
    init: ElementInitValue,
    doc: Option<String>,
    occurs: Option<Occurs>,
}

#[derive(Debug, PartialEq)]
pub enum ElementType {
    Named { name: String, type_: String },
}

impl Packet {
    pub fn new() -> Self {
        Packet {
            contents: Vec::new(),
            doc: None
        }
    }

    pub fn add_content(&mut self, content: PacketContent) {
        self.contents.push(content);
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn contents(&self) -> &[PacketContent] {
        &self.contents
    }

    pub fn into_contents(self) -> Vec<PacketContent> {
        self.contents
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

impl Sequence {
    pub fn new(doc: Option<String>) -> Self {
        Sequence {
            contents: Vec::new(),
            doc
        }
    }

    pub fn add_content(&mut self, content: SequenceContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[SequenceContent] {
        &self.contents
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }
}

impl Choice {
    pub fn new(doc: Option<String>) -> Self {
        Choice {
            contents: Vec::new(),
            doc: doc
        }
    }

    pub fn add_content(&mut self, content: SequenceContent) {
        self.contents.push(content);
    }

    pub fn contents(&self) -> &[SequenceContent] {
        &self.contents
    }

    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn set_doc(&mut self, doc: String) {
        self.doc = Some(doc);
    }
}

impl Element {
    pub fn new(type_: ElementType, init: ElementInitValue,
                occurs: Option<Occurs>) -> Self {
        Element {
            type_,
            occurs,
            doc: None,
            init,
        }
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
}

impl SimpleType {
    pub fn new(name: String, content: SimpleTypeContent, doc: Option<String>) -> Self {
        SimpleType {
            name,
            content,
            doc
        }
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
