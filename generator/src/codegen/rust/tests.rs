//! What the Rust backend writes for each construct of the XML. The encoding
//! rules themselves are checked end to end where the output is used (against
//! the C++ backend's code); these tests pin down that each XML construct maps
//! to the rule meant for it.

use std::collections::BTreeMap;

use super::{generate_files, RustArgs};

fn args() -> RustArgs {
    RustArgs {
        output_folder: String::new(),
        shared_types_path: "crate::packets::shared_types".into(),
        runtime_path: "crate::wire".into(),
        consts_path: "crate::consts".into(),
        extern_type: vec!["Isc::ServerType=crate::external::IscServerType".into()],
        samples: Some("crate::testgen".into()),
        samples_feature: "testgen".into(),
    }
}

/// Generates the given XML documents: packets, or definitions.
fn generate_with(xmls: &[&str], args: &RustArgs) -> Result<BTreeMap<String, String>, String> {
    let (mut packets, mut definitions) = (Vec::new(), Vec::new());
    for xml in xmls {
        match ::schema::Reader::load_document(std::io::Cursor::new(xml.as_bytes().to_vec())).unwrap() {
            ::schema::ast::Document::Packet(packet) => {
                let packet = ::flatten::flatten(std::path::Path::new("."), &packet).unwrap();
                packets.push(::graph_passes::run(packet).unwrap());
            }
            ::schema::ast::Document::Definitions(d) => definitions.push(::flatten::flatten_definitions(&d)),
        }
    }
    generate_files("test", &packets, &definitions, args)
}

fn generate(xmls: &[&str]) -> BTreeMap<String, String> {
    generate_with(xmls, &args()).unwrap_or_else(|e| panic!("{}", e))
}

/// Asserts that `haystack` contains each of `needles`, in that order.
fn contains_in_order(haystack: &str, needles: &[&str]) {
    let mut rest = haystack;
    for needle in needles {
        match rest.find(needle) {
            Some(at) => rest = &rest[at + needle.len()..],
            None => panic!("{:?} not found (in order) in:\n{}", needle, haystack),
        }
    }
}

const LOGIN: &str = r#"<packet ePacketType = "PAKCS_LOGIN_REQ">
    <element name = "password" type = "uint8_t" occurs = "32" />
    <element name = "username" type = "std::string" default = "" />
    <element name = "level" type = "uint16_t" default = "7" />
</packet>"#;

#[test]
fn a_packet_is_a_struct_with_its_type_and_name() {
    let files = generate(&[LOGIN]);
    let packet = &files["cli_login_req.rs"];
    contains_in_order(
        packet,
        &[
            "use crate::wire::{BString, Packet, PacketType, Reader, Wire, WireError, Writer};",
            "use crate::packets::shared_types::*;",
            "pub struct CliLoginReq {",
            "pub password: [u8; 32],",
            "pub username: BString,",
            "pub level: u16,",
            "level: 7 as u16,",
            "for item in &self.password {",
            "self.username.write(w);",
            "let password = r.read_array()?;",
            "const TYPE: PacketType = PacketType::PAKCS_LOGIN_REQ;",
            "const NAME: &'static str = \"CliLoginReq\";",
        ],
    );
    assert!(!packet.contains("bincode"));
    let index = &files["mod.rs"];
    contains_in_order(index, &["pub mod shared_types;", "pub mod cli_login_req;", "cli_login_req::CliLoginReq,"]);
}

#[test]
fn a_list_without_a_count_runs_to_the_end_and_one_with_a_count_is_prefixed() {
    let files = generate(&[r#"<packet ePacketType = "PAKWC_LIST">
        <element name = "count_first" type = "uint32_t" occurs = "unbounded" occursSize = "uint8_t" />
        <element name = "rest" type = "uint16_t" occurs = "unbounded" />
    </packet>"#]);
    contains_in_order(
        &files["srv_list.rs"],
        &[
            "pub count_first: Vec<u32>,",
            "pub rest: Vec<u16>,",
            "(self.count_first.len() as u8).write(w);",
            "let count = <u8 as Wire>::read(r)? as usize;",
            "let count_first = r.read_counted(count)?;",
            "let rest = r.read_to_end();",
        ],
    );
}

#[test]
fn a_list_without_a_count_must_be_last() {
    let error = generate_with(
        &[r#"<packet ePacketType = "PAKWC_LIST">
            <element name = "rest" type = "uint16_t" occurs = "unbounded" />
            <element name = "after" type = "uint8_t" />
        </packet>"#],
        &args(),
    )
    .unwrap_err();
    assert!(error.contains("must be the last element"), "{}", error);
}

#[test]
fn bit_fields_are_packed_low_bit_first_into_whole_bytes() {
    let files = generate(&[r#"<packet ePacketType = "PAKWC_BITS">
        <element name = "a" type = "uint8_t" bits = "3" />
        <element name = "b" type = "uint16_t" bits = "13" />
        <element name = "after" type = "uint8_t" />
    </packet>"#]);
    contains_in_order(
        &files["srv_bits.rs"],
        &[
            "let mut bits = 0u64;",
            "bits |= (self.a as u64 & 0x7) << 0;",
            "bits |= (self.b as u64 & 0x1fff) << 3;",
            "w.put_bits(bits, 2);",
            "self.after.write(w);",
            "let bits = r.get_bits(2)?;",
            "let b = ((bits >> 3) & 0x1fff) as u16;",
        ],
    );
}

const ITEM: &str = r#"<packet ePacketType = "PAKWC_ITEM">
    <complexType name = "Header">
        <choice>
            <sequence inline = "true">
                <element name = "type" type = "uint16_t" bits = "5" />
                <element name = "id" type = "uint16_t" bits = "10" />
                <element name = "isCreated" type = "uint16_t" bits = "1" />
            </sequence>
            <element name = "header" type = "uint16_t" />
        </choice>
    </complexType>
    <simpleType name = "Kind">
        <restriction base = "uint8_t">
            <enumeration value = "NONE" id = "0" />
            <enumeration value = "BIG_ONE" />
            <enumeration value = "LAST" id = "9" />
        </restriction>
    </simpleType>
    <simpleType name = "Name">
        <restriction base = "std::string">
            <length value = "16" />
        </restriction>
    </simpleType>
    <element name = "header" type = "Header" />
    <element name = "kind" type = "Kind" />
    <element name = "name" type = "Name" />
</packet>"#;

#[test]
fn a_choice_is_stored_as_its_widest_unsigned_integer_with_views_onto_it() {
    let files = generate(&[ITEM]);
    contains_in_order(
        &files["shared_types.rs"],
        &[
            "pub struct Header(pub u16);",
            "pub fn type_(&self) -> u16 {",
            "let bits = (self.0 >> 0) & (0x1f as u16);",
            "pub fn set_type(&mut self, value: u16) {",
            "pub fn id(&self) -> u16 {",
            "let bits = (self.0 >> 5) & (0x3ff as u16);",
            "pub fn is_created(&self) -> u16 {",
            "let bits = (self.0 >> 15) & (0x1 as u16);",
            "pub fn header(&self) -> u16 {",
        ],
    );
}

#[test]
fn an_enumeration_keeps_unknown_values_and_a_length_fixes_a_string() {
    let files = generate(&[ITEM]);
    let shared = &files["shared_types.rs"];
    contains_in_order(
        shared,
        &[
            "pub struct Kind(pub u8);",
            "pub const NONE: Self = Self((0) as u8);",
            // Without an id, an enumerator follows the previous one.
            "pub const BIG_ONE: Self = Self((1) as u8);",
            "pub const LAST: Self = Self((9) as u8);",
            "None => write!(f, \"Kind({})\", self.0),",
        ],
    );
    contains_in_order(
        shared,
        &[
            "pub struct Name(pub BString);",
            "pub const LEN: usize = 16;",
            "w.put_bytes(&self.0[..self.0.len().min(Self::LEN)]);",
            "Ok(Self(BString::from(r.take(Self::LEN)?)))",
        ],
    );
    // The packet module offers the types of its own fields.
    assert!(files["srv_item.rs"].contains("pub use crate::packets::shared_types::{Header, Kind, Name};"));
}

#[test]
fn types_are_generated_once_and_only_when_used() {
    let other = r#"<packet ePacketType = "PAKWC_OTHER">
        <simpleType name = "Unused"><restriction base = "uint8_t" /></simpleType>
        <simpleType name = "Kind">
            <restriction base = "uint8_t">
                <enumeration value = "NONE" id = "0" />
                <enumeration value = "BIG_ONE" />
                <enumeration value = "LAST" id = "9" />
            </restriction>
        </simpleType>
        <element name = "kind" type = "Kind" />
    </packet>"#;
    let files = generate(&[ITEM, other]);
    let shared = &files["shared_types.rs"];
    assert_eq!(shared.matches("pub struct Kind(").count(), 1);
    assert!(!shared.contains("Unused"));
}

#[test]
fn counts_come_from_numbers_constants_and_enumerators() {
    let files = generate(&[r#"<packet ePacketType = "PAKWC_COUNTS">
        <simpleType name = "Slots">
            <restriction base = "uint8_t">
                <enumeration value = "First" id = "0" />
                <enumeration value = "MaxSlots" id = "3" />
            </restriction>
        </simpleType>
        <element name = "a" type = "uint8_t" occurs = "0x4" />
        <element name = "b" type = "uint8_t" occurs = "MAX_ITEMS" />
        <element name = "c" type = "uint8_t" occurs = "Slots::MaxSlots" />
    </packet>"#]);
    contains_in_order(
        &files["srv_counts.rs"],
        &["pub a: [u8; 4],", "pub b: [u8; crate::consts::MAX_ITEMS as usize],", "pub c: [u8; 3],"],
    );
}

#[test]
fn an_external_enum_needs_a_mapping() {
    let server = r#"<packet ePacketType = "ISC_SERVER_REGISTER">
        <element name = "type" type = "Isc::ServerType" enum = "uint8_t" />
    </packet>"#;
    let files = generate(&[server]);
    assert!(files["isc_server_register.rs"].contains("pub type_: crate::external::IscServerType,"));

    let unmapped = RustArgs {
        extern_type: Vec::new(),
        ..args()
    };
    let error = generate_with(&[server], &unmapped).unwrap_err();
    assert!(error.contains("--extern-type Isc::ServerType"), "{}", error);
}

#[test]
fn samples_are_optional_and_behind_their_feature() {
    let files = generate(&[LOGIN]);
    contains_in_order(
        &files["cli_login_req.rs"],
        &[
            "#[cfg(feature = \"testgen\")]\nuse crate::testgen::{Gen, Sample};",
            "#[cfg(feature = \"testgen\")]\nimpl Sample for CliLoginReq {",
        ],
    );
    let without = RustArgs {
        samples: None,
        ..args()
    };
    let files = generate_with(&[LOGIN], &without).unwrap();
    assert!(!files["cli_login_req.rs"].contains("Sample"));
}

const PACKET_TYPE: &str = r#"<definitions name = "packet_type">
    <documentation>Packet type ids.</documentation>
    <simpleType name = "ePacketType" scoped = "true">
        <restriction base = "uint16_t">
            <enumeration value = "PAKSTART" id = "0x700" />
            <enumeration value = "PAKCS_ALIVE" id = "0x700" />
            <enumeration value = "PAKSS_ERROR" />
            <enumeration value = "STRESS" id = "0x6F6D" />
        </restriction>
    </simpleType>
    <constant name = "MAX_STAT" type = "uint32_t" value = "300">
        <documentation>Highest a stat goes.</documentation>
    </constant>
    <constant name = "TAB_SIZE" type = "uint8_t" value = "0x1E" />
    <constant name = "DROP_RANGE" type = "float" value = "50" />
</definitions>"#;

#[test]
fn definitions_are_a_module_of_their_own() {
    let files = generate(&[LOGIN, PACKET_TYPE]);
    contains_in_order(&files["mod.rs"], &["pub mod shared_types;", "pub mod packet_type;", "pub mod cli_login_req;"]);
    let module = &files["packet_type.rs"];
    contains_in_order(
        module,
        &[
            "//! Packet type ids.",
            "pub struct EPacketType(pub u16);",
            "pub const PAKSTART: Self = Self((1792) as u16);",
            "pub const PAKCS_ALIVE: Self = Self((1792) as u16);",
            "pub const PAKSS_ERROR: Self = Self((1793) as u16);",
            "pub const STRESS: Self = Self((28525) as u16);",
            // The packet, not the marker before it, names the value.
            "1792 => Some(\"PAKCS_ALIVE\"),",
            "/// Highest a stat goes.\npub const MAX_STAT: u32 = 300;",
            "pub const TAB_SIZE: u8 = 0x1E;",
            "pub const DROP_RANGE: f32 = 50.0;",
        ],
    );
    // Definitions are not types of the packets.
    assert!(!files["shared_types.rs"].contains("EPacketType"));
}

#[test]
fn a_constant_has_to_fit_its_type() {
    let bad = |ty: &str, value: &str| {
        let xml = format!(r#"<definitions name = "consts"><constant name = "X" type = "{}" value = "{}" /></definitions>"#, ty, value);
        generate_with(&[xml.as_str()], &args()).unwrap_err()
    };
    assert!(bad("uint8_t", "256").contains("does not fit"));
    assert!(bad("uint32_t", "-1").contains("does not fit"));
    assert!(bad("int32_t", "1.5").contains("is not an integer"));
    assert!(bad("float", "fast").contains("is not a number"));
    assert!(bad("std::string", "x").contains("is not an integer, float or double"));
}

#[test]
fn a_scoped_enumeration_belongs_in_definitions() {
    let xml = r#"<packet ePacketType = "PAKWC_SCOPED">
        <simpleType name = "Kind" scoped = "true">
            <restriction base = "uint8_t"><enumeration value = "A" /></restriction>
        </simpleType>
        <element name = "kind" type = "Kind" />
    </packet>"#;
    let packet = ::schema::Reader::load_packet(std::io::Cursor::new(xml.as_bytes().to_vec())).unwrap();
    let error = ::flatten::flatten(std::path::Path::new("."), &packet).unwrap_err();
    assert!(error.to_string().contains("only supported in <definitions>"), "{}", error);
}
