//! What the Rust backend writes for a `<choice selector="...">`: an enum
//! whose variant decides both the bytes that follow the selector and the
//! value written for it.

use super::tests::{args, contains_in_order, generate, generate_with};

/// A clan reply: the result picks what follows. `pub(super)` for the round
/// trip test, which compiles it.
pub(super) const CLAN: &str = r#"<packet ePacketType = "PAKCC_CLAN_COMMAND">
    <simpleType name = "ClanResult">
        <restriction base = "uint8_t">
            <enumeration value = "CREATED" id = "0x30" />
            <enumeration value = "NAME_TAKEN" />
            <enumeration value = "ROSTER" />
            <enumeration value = "MOTD" />
        </restriction>
    </simpleType>
    <complexType name = "MyClan">
        <sequence>
            <element name = "id" type = "uint32_t" />
            <element name = "level" type = "uint8_t" />
        </sequence>
    </complexType>
    <complexType name = "ClanMember">
        <sequence>
            <element name = "name" type = "std::string" />
            <element name = "level" type = "uint16_t" />
        </sequence>
    </complexType>
    <element name = "result" type = "ClanResult" />
    <choice selector = "result">
        <sequence case = "CREATED">
            <element name = "clan" type = "MyClan" />
            <element name = "name" type = "std::string" />
        </sequence>
        <sequence case = "NAME_TAKEN" />
        <sequence case = "ROSTER">
            <element name = "members" type = "ClanMember" occurs = "unbounded" />
        </sequence>
    </choice>
</packet>"#;

/// An integer selector with an element between it and the choice, and a
/// branch for every other value.
pub(super) const EVENT: &str = r#"<packet ePacketType = "PAKWC_EVENT">
    <element name = "kind" type = "uint16_t" />
    <element name = "time" type = "uint32_t" />
    <choice selector = "kind" name = "event">
        <sequence case = "1">
            <element name = "x" type = "float" />
            <element name = "tags" type = "uint8_t" occurs = "unbounded" occursSize = "uint8_t" />
        </sequence>
        <sequence case = "0x10" />
        <sequence default = "true">
            <element name = "raw" type = "uint8_t" occurs = "unbounded" />
        </sequence>
    </choice>
</packet>"#;

/// A selector choice inside a named type, which packets share.
pub(super) const IN_A_TYPE: &str = r#"<packet ePacketType = "PAKWC_SHAPE">
    <complexType name = "Shape">
        <sequence>
            <element name = "kind" type = "int8_t" />
            <choice selector = "kind">
                <sequence case = "-1" />
                <sequence case = "2">
                    <element name = "w" type = "uint16_t" />
                    <element name = "h" type = "uint16_t" />
                </sequence>
            </choice>
            <element name = "after" type = "uint8_t" />
        </sequence>
    </complexType>
    <element name = "shape" type = "Shape" />
    <element name = "id" type = "uint32_t" />
</packet>"#;

/// Enumeration selectors with default branches, in a type and in a packet.
pub(super) const ENUM_DEFAULT: &str = r#"<packet ePacketType = "PAKWC_WISH_LIST">
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
</packet>"#;

#[test]
fn the_selector_is_no_field_of_its_own_the_variant_decides_it() {
    let files = generate(&[CLAN]);
    let packet = &files["srv_clan_command.rs"];
    contains_in_order(
        packet,
        &[
            "pub use crate::packets::shared_types::{ClanMember, ClanResult, MyClan, SrvClanCommandBody};",
            "pub struct SrvClanCommand {",
            "pub body: SrvClanCommandBody,",
            "}",
            // The bytes for the selector come from the variant.
            "self.body.selector().write(w);",
            "self.body.write_case(w);",
            "let result: ClanResult = Wire::read(r)?;",
            "let body = SrvClanCommandBody::read_case(result, r)?;",
            "Ok(Self { body })",
        ],
    );
    assert!(!packet.contains("pub result"), "{}", packet);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "pub fn selector(&self) -> ClanResult {",
            "Self::Created { .. } => ClanResult::CREATED,",
            "Self::NameTaken => ClanResult::NAME_TAKEN,",
            "Self::Roster { .. } => ClanResult::ROSTER,",
        ],
    );
}

#[test]
fn a_case_is_a_variant_with_the_branch_s_fields() {
    let files = generate(&[CLAN]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "#[derive(Debug, Clone, PartialEq)]\npub enum SrvClanCommandBody {",
            "    Created {\n        clan: MyClan,\n        name: BString,\n    },",
            "pub fn write_case(&self, w: &mut Writer) {",
            "Self::Created { clan, name } => {",
            "(*clan).write(w);",
            "(*name).write(w);",
            "pub fn read_case(selector: ClanResult, r: &mut Reader<'_>) -> Result<Self, WireError> {",
            "ClanResult::CREATED => {",
            "let clan = Wire::read(r)?;",
            "let name = Wire::read(r)?;",
            "Ok(Self::Created { clan, name })",
        ],
    );
}

#[test]
fn an_empty_case_is_a_unit_variant() {
    let files = generate(&[CLAN]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "    NameTaken,\n",
            "Self::NameTaken => {}",
            "ClanResult::NAME_TAKEN => Ok(Self::NameTaken),",
        ],
    );
}

#[test]
fn a_list_to_the_end_of_the_packet_can_end_a_branch() {
    let files = generate(&[CLAN]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "    Roster {\n        members: Vec<ClanMember>,\n    },",
            "Self::Roster { members } => {",
            "for item in &(*members) {",
            "ClanResult::ROSTER => {",
            "let members = r.read_to_end();",
            "Ok(Self::Roster { members })",
        ],
    );
}

#[test]
fn an_unknown_value_without_a_default_is_a_read_error() {
    let files = generate(&[CLAN]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "pub fn read_case(",
            "_ => Err(WireError::unknown_case(\"SrvClanCommandBody\", selector.0 as i64)),",
        ],
    );
}

#[test]
fn a_default_branch_keeps_the_value_it_was_read_with() {
    let files = generate(&[EVENT]);
    let shared = &files["shared_types.rs"];
    contains_in_order(
        shared,
        &[
            "pub enum SrvEventEvent {",
            "    Case1 {\n        x: f32,\n        tags: Vec<u8>,\n    },",
            "    Case16,\n",
            "    Other {\n        kind: u16,\n        raw: Vec<u8>,\n    },",
            "pub fn selector(&self) -> u16 {",
            "Self::Case1 { .. } => 1,",
            "Self::Case16 => 16,",
            "Self::Other { kind, .. } => *kind,",
            "Self::Other { raw, .. } => {",
            "pub fn read_case(selector: u16, r: &mut Reader<'_>) -> Result<Self, WireError> {",
            "1 => {",
            "let count = <u8 as Wire>::read(r)? as usize;",
            "16 => Ok(Self::Case16),",
            "_ => {",
            "let kind = selector;",
            "let raw = r.read_to_end();",
            "Ok(Self::Other { kind, raw })",
        ],
    );
    assert!(!shared.contains("unknown_case"), "{}", shared);
    // A sampled default never carries the value of a named case: it would
    // be read back as that case.
    contains_in_order(
        shared,
        &[
            "impl Sample for SrvEventEvent {",
            "match g.below(3) {",
            "let value = g.next_u64() as u16;",
            "if !matches!(value, 1 | 16) {",
            "break value;",
        ],
    );
    contains_in_order(
        &files["srv_event.rs"],
        &[
            "pub time: u32,",
            "pub event: SrvEventEvent,",
            "self.event.selector().write(w);",
            "self.time.write(w);",
            "self.event.write_case(w);",
            "let kind: u16 = Wire::read(r)?;",
            "let time = Wire::read(r)?;",
            "let event = SrvEventEvent::read_case(kind, r)?;",
            "Ok(Self { time, event })",
        ],
    );
}

#[test]
fn a_type_can_hold_a_selector_choice() {
    let files = generate(&[IN_A_TYPE]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "pub struct Shape {",
            "pub body: ShapeBody,",
            "pub after: u8,",
            "let kind: i8 = Wire::read(r)?;",
            "let body = ShapeBody::read_case(kind, r)?;",
            "pub enum ShapeBody {",
            "CaseMinus1,",
            "Case2 {",
            "-1 => Ok(Self::CaseMinus1),",
        ],
    );
}

#[test]
fn a_selector_choice_has_to_be_well_formed() {
    let packet = |body: &str| {
        format!(
            r#"<packet ePacketType = "PAKWC_BAD">
                <simpleType name = "Kind">
                    <restriction base = "uint8_t">
                        <enumeration value = "A" />
                        <enumeration value = "B" />
                        <enumeration value = "ALSO_A" id = "0" />
                    </restriction>
                </simpleType>
                {}
            </packet>"#,
            body
        )
    };
    let cases = [
        (
            r#"<choice selector = "kind"><sequence case = "A" /></choice>
               <element name = "kind" type = "Kind" />"#,
            "not an earlier element",
        ),
        (
            r#"<element name = "kind" type = "std::string" />
               <choice selector = "kind"><sequence case = "A" /></choice>"#,
            "neither an enumeration",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind"><sequence case = "C" /></choice>"#,
            "is not a value of Kind",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind"><sequence case = "A" /><sequence case = "ALSO_A" /></choice>"#,
            "the same value",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind"><sequence default = "true" /><sequence default = "true" /></choice>"#,
            "more than one default",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind"><sequence /></choice>"#,
            "case=",
        ),
        (
            r#"<element name = "kind" type = "uint8_t" />
               <choice selector = "kind"><sequence case = "300" /></choice>"#,
            "does not fit",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind">
                   <sequence case = "A"><element name = "rest" type = "uint8_t" occurs = "unbounded" /></sequence>
               </choice>
               <element name = "after" type = "uint8_t" />"#,
            "must be the last element",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind">
                   <sequence case = "A"><element name = "a" type = "uint8_t" bits = "4" /><element name = "b" type = "uint8_t" bits = "4" /></sequence>
               </choice>"#,
            "bit fields",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind">
                   <sequence case = "A"><element name = "x" type = "uint8_t" /></sequence>
                   <sequence case = "B"><element name = "x" type = "uint16_t" /></sequence>
               </choice>"#,
            "same type",
        ),
        (
            r#"<element name = "kind" type = "Kind" />
               <choice selector = "kind">
                   <sequence case = "A"><element name = "kind" type = "uint8_t" /></sequence>
               </choice>"#,
            "already an element",
        ),
    ];
    for (body, wanted) in cases {
        let error = generate_with(&[packet(body).as_str()], &args()).unwrap_err();
        assert!(error.contains(wanted), "wanted {:?}, got: {}", wanted, error);
    }
}

#[test]
fn two_packets_get_two_enums() {
    let other = CLAN.replace("PAKCC_CLAN_COMMAND", "PAKCS_CLAN_COMMAND");
    let files = generate(&[CLAN, other.as_str()]);
    let shared = &files["shared_types.rs"];
    assert_eq!(shared.matches("pub enum SrvClanCommandBody {").count(), 1, "{}", shared);
    assert_eq!(shared.matches("pub enum CliClanCommandBody {").count(), 1, "{}", shared);
}

#[test]
fn an_enumeration_selector_can_have_a_default_branch() {
    let files = generate(&[ENUM_DEFAULT]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "pub enum EventBody {",
            "    Other {\n        kind: Kind,\n        raw: u16,\n    },",
            "pub enum SrvWishListBody {",
            "    Two {\n        y: u8,\n    },",
            "    Other {\n        result: Kind,\n    },",
            "Self::Other { result, .. } => *result,",
            "Self::Other { .. } => {}",
            "_ => Ok(Self::Other { result: selector }),",
            "let value = Kind(g.next_u64() as u8);",
            "if !matches!(value, Kind::TWO) {",
        ],
    );
}
