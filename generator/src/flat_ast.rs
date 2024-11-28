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
    anonymous: bool,
    inline: bool
}

#[derive(Debug)]
pub enum ComplexTypeContent {
    Seq(Sequence),
    Choice(Choice),
    Empty
}

pub use ::schema::ast::Occurs;

#[derive(Debug, Clone)]
pub struct Sequence {
    elements: Vec<Element>,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    inline: bool
}

#[derive(Debug)]
pub struct Choice {
    elements: Vec<Element>,
    doc: Option<String>,
    occurs: Option<Occurs>,
    size_occurs: Option<String>,
    inline_seqs: ::std::collections::HashMap<String, Sequence>
}

#[derive(Debug, Clone)]
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
    is_defined: bool,
    special_read_write: Option<String>,
    bits: Option<u32>,
    occur_is_defined: bool,
    bitset: Option<Bitset>
}

#[derive(Debug, Clone)]
pub struct Bitset {
    pub size: u32,
    pub start: u32,
    pub name: String
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

pub use ::schema::ast::ElementInitValue;

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
    #[allow(dead_code)]
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
        let name = type_.clone().to_lower_camel_case();
        let (class_name, filename) = if name.starts_with("isc") {
            (name.clone(),
             name.clone().to_snake_case())
        } else {
            if name.starts_with("pakcs") {
                let name = "Cli".to_string() + &name[5..];
                (name.clone(),
                 name.clone().to_snake_case())
            } else {
                let name = "srv".to_string() + &name[5..];
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
        anonymous: bool,
        inline: bool
    ) -> Self {
        ComplexType{ name, content, doc, anonymous, inline }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    #[allow(dead_code)]
    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    pub fn content(&self) -> &ComplexTypeContent {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut ComplexTypeContent {
        &mut self.content
    }

    #[allow(dead_code)]
    pub fn anonymous(&self) -> bool {
        self.anonymous
    }

    pub fn inline(&self) -> bool {
        self.inline
    }
}

impl Sequence {
    pub fn new(
        occurs: Option<Occurs>,
        size_occurs: Option<String>,
        doc: Option<String>,
        inline: bool
    ) -> Self {
        Sequence{ elements: Vec::new(), occurs, size_occurs, doc, inline }
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

    pub fn inline(&self) -> bool {
        self.inline
    }

    #[allow(dead_code)]
    pub fn set_inline(&mut self, inline: bool) {
        self.inline = inline;
    }
}

impl Choice {
    pub fn new( occurs: Option<Occurs>, size_occurs: Option<String>
              , doc: Option<String>) -> Self {
        Choice{ elements: Vec::new(), occurs, size_occurs, doc, inline_seqs: ::std::collections::HashMap::new() }
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

    pub fn occurs(&self) -> &Option<Occurs> {
        &self.occurs
    }

    pub fn size_occurs(&self) -> &Option<String> {
        &self.size_occurs
    }

    pub fn inline_seqs(&self) -> &::std::collections::HashMap<String, Sequence> {
        &self.inline_seqs
    }

    pub fn add_inline_seqs(&mut self, name: String, seq: Sequence) {
        self.inline_seqs.insert(name, seq);
    }
}

impl Element {
    pub fn new(name: String, type_: String, id: u32
               , init: ElementInitValue
               , occurs: Option<Occurs>
               , size_occurs: Option<String>
               , doc: Option<String>
               , anonymous: bool
               , reference: bool
               , special_read_write: Option<String>
               , bits: Option<u32>
               , bitset: Option<Bitset>) -> Self {
        Element{ name, init, type_, id, occurs, size_occurs, doc
                 , anonymous, reference, enum_type: None,
                 is_defined: false, special_read_write, bits,
                 occur_is_defined: false, bitset }
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

    pub fn read_write(&self) -> &Option<String> {
        &self.special_read_write
    }

    #[allow(dead_code)]
    pub fn set_read_write(&mut self, read_write: String) {
        self.special_read_write = Some(read_write);
    }

    pub fn bits(&self) -> Option<u32> {
        self.bits
    }

    #[allow(dead_code)]
    pub fn set_bits(&mut self, bits: u32) {
        self.bits = Some(bits);
    }

    pub fn bitset(&self) -> &Option<Bitset> {
        &self.bitset
    }

    pub fn bitset_mut(&mut self) -> &mut Option<Bitset> {
        &mut self.bitset
    }

    #[allow(dead_code)]
    pub fn set_bitset(&mut self, bitset: Bitset) {
        self.bitset = Some(bitset);
    }

    pub fn set_occur_is_defined(&mut self) {
        self.occur_is_defined = true;
    }

    pub fn occur_is_defined(&self) -> bool {
        self.occur_is_defined
    }
}

impl Bitset {
    pub fn new(size: u32, start: u32, name: String) -> Self {
        Self {size, start, name}
    }
}

impl SimpleType {
    pub fn new(name: String, doc: Option<String>) -> Self {
        use heck::ToLowerCamelCase;
        SimpleType{ name: name.to_lower_camel_case(), contents: Vec::new(), doc }
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
