//! What the C++ backend writes for a `<choice selector="...">`: the selector
//! stays a member, the members of every branch are members too, and reading
//! and writing switch on the selector, so the bytes always follow it.

use crate::writer::Writer;
use super::{codegen_header, codegen_source};

struct Text(String);

impl std::io::Write for Text {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 += std::str::from_utf8(buf).unwrap();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn packet(xml: &str) -> crate::flat_ast::Packet {
    let packet = ::schema::Reader::load_packet(std::io::Cursor::new(xml.as_bytes().to_vec())).unwrap();
    let packet = ::flatten::flatten(std::path::Path::new("."), &packet).unwrap();
    ::graph_passes::run(packet).unwrap()
}

/// The generated header and source.
fn generate(xml: &str) -> (String, String) {
    let packet = packet(xml);
    let mut header = Writer::new(Text(String::new()));
    codegen_header::CodeHeaderGenerator::new(&mut header, "0".into()).generate(&packet).unwrap();
    let mut source = Writer::new(Text(String::new()));
    codegen_source::CodeSourceGenerator::new(&mut source).generate(&packet).unwrap();
    (header.into().0, source.into().0)
}

fn contains_in_order(haystack: &str, needles: &[&str]) {
    let mut rest = haystack;
    for needle in needles {
        match rest.find(needle) {
            Some(at) => rest = &rest[at + needle.len()..],
            None => panic!("{:?} not found (in order) in:\n{}", needle, haystack),
        }
    }
}

const CLAN: &str = r#"<packet ePacketType = "PAKCC_CLAN_COMMAND">
    <simpleType name = "ClanResult">
        <restriction base = "uint8_t">
            <enumeration value = "CREATED" id = "0x30" />
            <enumeration value = "NAME_TAKEN" />
            <enumeration value = "ROSTER" />
            <enumeration value = "MOTD" />
        </restriction>
    </simpleType>
    <complexType name = "ClanMember">
        <sequence>
            <element name = "name" type = "std::string" />
            <element name = "level" type = "uint16_t" />
        </sequence>
    </complexType>
    <element name = "result" type = "ClanResult" />
    <choice selector = "result">
        <sequence case = "CREATED">
            <element name = "id" type = "uint32_t" />
            <element name = "name" type = "std::string" />
        </sequence>
        <sequence case = "NAME_TAKEN" />
        <sequence case = "ROSTER">
            <element name = "name" type = "std::string" />
            <element name = "members" type = "ClanMember" occurs = "unbounded" />
        </sequence>
    </choice>
</packet>"#;

#[test]
fn every_branch_member_is_a_member_once() {
    let (header, _) = generate(CLAN);
    contains_in_order(
        &header,
        &[
            "SrvClanCommand& set_result(const ClanResult);",
            "SrvClanCommand& set_id(const uint32_t);",
            "SrvClanCommand& set_name(const std::string);",
            "SrvClanCommand& set_members(const std::vector<ClanMember>);",
            "static SrvClanCommand create(const ClanResult& result);",
            "private:",
            "ClanResult result;",
            "uint32_t id;",
            "std::string name;",
            "std::vector<ClanMember> members;",
        ],
    );
    // Once in ClanMember, once in the packet though two branches have it.
    assert_eq!(header.matches("std::string name;").count(), 2, "{}", header);
    assert!(!header.contains("SrvClanCommandBody"), "{}", header);
}

#[test]
fn reading_takes_exactly_the_branch_of_the_selector() {
    let (_, source) = generate(CLAN);
    contains_in_order(
        &source,
        &[
            "SrvClanCommand::SrvClanCommand(CRoseReader reader) : CRosePacket(reader) {",
            "result = static_cast<ClanResult>(result_temp);",
            "switch (result) {",
            "case ClanResult::CREATED: {",
            "if (!reader.get_uint32_t(id)) {",
            "if (!reader.get_string(name)) {",
            "break;",
            "case ClanResult::NAME_TAKEN: {",
            "break;",
            "case ClanResult::ROSTER: {",
            "if (!reader.get_string(name)) {",
            // To the end of the packet.
            "while (reader.get_iserialize(elem)) {",
            "members.push_back(elem);",
            "break;",
            // An unknown value without a default branch is no packet.
            "default: {",
            "return;",
        ],
    );
}

#[test]
fn writing_follows_the_selector_and_refuses_an_unknown_one() {
    let (_, source) = generate(CLAN);
    contains_in_order(
        &source,
        &[
            "bool SrvClanCommand::pack(CRoseBasePolicy& writer) const {",
            "if (!writer.set_uint8_t(result)) {",
            "switch (result) {",
            "case ClanResult::CREATED: {",
            "if (!writer.set_uint32_t(id)) {",
            "case ClanResult::ROSTER: {",
            "for (const auto& elem : members) {",
            "default: {",
            "return false;",
            "return true;",
        ],
    );
    // A choice is no fixed size part of the packet.
    assert!(!source.contains("SrvClanCommandBody"), "{}", source);
}

#[test]
fn a_default_branch_and_integer_cases() {
    let (_, source) = generate(r#"<packet ePacketType = "PAKWC_EVENT">
        <element name = "kind" type = "uint16_t" />
        <choice selector = "kind">
            <sequence case = "0x10"><element name = "x" type = "float" /></sequence>
            <sequence default = "true"><element name = "raw" type = "uint8_t" occurs = "unbounded" /></sequence>
        </choice>
    </packet>"#);
    contains_in_order(
        &source,
        &[
            "switch (kind) {",
            "case 16: {",
            "if (!reader.get_float(x)) {",
            "default: {",
            "while (reader.get_uint8_t(elem)) {",
            "switch (kind) {",
            "case 16: {",
            "default: {",
            "for (const auto& elem : raw) {",
        ],
    );
}

#[test]
fn a_type_can_hold_a_selector_choice() {
    let (header, source) = generate(r#"<packet ePacketType = "PAKWC_SHAPE">
        <complexType name = "Shape">
            <sequence>
                <element name = "kind" type = "int8_t" />
                <choice selector = "kind">
                    <sequence case = "-1" />
                    <sequence case = "2">
                        <element name = "w" type = "uint16_t" />
                    </sequence>
                </choice>
                <element name = "after" type = "uint8_t" />
            </sequence>
        </complexType>
        <element name = "shape" type = "Shape" />
    </packet>"#);
    contains_in_order(&header, &["struct Shape : public ISerialize {", "Shape& set_w(const uint16_t);", "uint16_t w;", "uint8_t after;"]);
    contains_in_order(
        &source,
        &[
            "bool SrvShape::Shape::write(CRoseBasePolicy& writer) const {",
            "switch (kind) {",
            "case -1: {",
            "case 2: {",
            "if (!writer.set_uint16_t(w)) {",
            "default: {",
            "return false;",
            "if (!writer.set_uint8_t(after)) {",
            "bool SrvShape::Shape::read(CRoseReader& reader) {",
            "switch (kind) {",
            "if (!reader.get_uint16_t(w)) {",
            "default: {",
            "return false;",
            "if (!reader.get_uint8_t(after)) {",
        ],
    );
    assert!(!source.contains("ShapeBody"), "{}", source);
}

#[test]
fn an_enumeration_selector_in_a_type_is_read_and_written_as_its_integer() {
    let (_, source) = generate(r#"<packet ePacketType = "PAKWC_WISH_LIST">
    <simpleType name = "Kind">
        <restriction base = "uint8_t">
            <enumeration value = "NONE" />
            <enumeration value = "ONE" />
            <enumeration value = "TWO" />
        </restriction>
    </simpleType>
    <complexType name = "Event">
        <sequence>
            <element name = "kind" type = "Kind" />
            <choice selector = "kind">
                <sequence case = "ONE">
                    <element name = "x" type = "Kind" />
                </sequence>
                <sequence default = "true">
                    <element name = "raw" type = "uint16_t" />
                </sequence>
            </choice>
        </sequence>
    </complexType>
    <element name = "event" type = "Event" />
    <element name = "result" type = "Kind" />
    <choice selector = "result">
        <sequence case = "TWO">
            <element name = "y" type = "uint8_t" />
        </sequence>
        <sequence default = "true" />
    </choice>
</packet>"#);
    contains_in_order(
        &source,
        &[
            "bool SrvWishList::Event::write(CRoseBasePolicy& writer) const {",
            "if (!writer.set_uint8_t(kind)) {",
            "switch (kind) {",
            "case Kind::ONE: {",
            "if (!writer.set_uint8_t(x)) {",
            "bool SrvWishList::Event::read(CRoseReader& reader) {",
            "uint8_t kind_temp;",
            "kind = static_cast<Kind>(kind_temp);",
            "switch (kind) {",
            "x = static_cast<Kind>(x_temp);",
            "default: {",
            "if (!reader.get_uint16_t(raw)) {",
        ],
    );
}
