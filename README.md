# IDL
packet generator for osiROSE-new

##### Build status

Check the actions tab for build status.
##### Rust backend

```sh
packet_generator -i a.xml -i b.xml ... rust --output-folder src/packets \
    --runtime-path crate::wire \
    --shared-types-path crate::packets::shared_types \
    --consts-path crate::consts \
    --extern-type Isc::ServerType=crate::external::IscServerType \
    [--samples crate::testgen [--samples-feature testgen]]
```

All packets are generated in one run: a module per packet, `shared_types.rs`
with the types they use, and `mod.rs` with a `for_each_packet!` macro. The
generated code encodes exactly as the C++ backend's does and calls a small
runtime the using crate provides at `--runtime-path`; see
`generator/src/codegen/rust/mod.rs` for the rules and the runtime it expects.

##### Definitions

Enumerations and constants that are not part of any packet, such as the
packet type ids, go in a file of their own:

```xml
<definitions name = "packet_type">
    <documentation>Packet type ids.</documentation>
    <simpleType name = "ePacketType" scoped = "true">
        <restriction base = "uint16_t">
            <enumeration value = "PAKSTART" id = "0x500" />
            <enumeration value = "PAKCS_ALIVE" id = "0x500" />
            <enumeration value = "PAKSS_ERROR" />
        </restriction>
    </simpleType>
    <constant name = "MAX_STAT" type = "uint32_t" value = "300" />
    <constant name = "DROP_RANGE" type = "float" value = "50" />
</definitions>
```

- `name` names the generated files: `packet_type.h` and `packet_type.cpp`
  (C++; the source only includes the header), `packet_type.rs` (Rust, listed
  in `mod.rs` beside `shared_types`).
- A `simpleType` has to be an enumeration. `scoped = "true"` makes it an
  `enum class` in C++ (only in definitions: packets read and write their
  enumerations as plain integers). In C++ it keeps its
  name as written; in Rust it is in upper camel case (`EPacketType`). Of
  several enumerators with one value, the last one names the value in Rust's
  `name()` and `Debug`.
- A `constant` has an integer type (`uint8_t` to `int64_t`), `float` or
  `double`, and a value of that type: decimal or `0x` for integers. It
  becomes `constexpr` in namespace `RoseCommon`, and a `pub const` of the
  matching Rust type. Constants used as element counts (`occurs`) are read
  from `--consts-path` and converted to `usize` there.

Pass definitions files with `-i` like the packets.

##### Selector choices

A `<choice>` with a `selector` holds exactly one of its branches: the one for
the value of an earlier element of the same packet or sequence.

```xml
<packet ePacketType = "PAKCC_CLAN_COMMAND">
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
        <sequence default = "true" />
    </choice>
</packet>
```

- The selector is a plain element (no list, bit field or count) of an
  enumeration of the XML or of an integer type. `case` is one of the
  enumerators, or an integer (decimal or `0x`, maybe negative); no two cases
  have the same value. At most one branch is `default = "true"`, for every
  other value; without one, any other value fails to read and to write.
- A branch is a `<sequence>` of elements, maybe none. Its last element may be
  a list without `occursSize`, running to the end of the packet, if the
  choice is the last element. Not in a branch: bit fields, `occursRef`,
  `occursBits`, nested sequences or choices. An element name used in several
  branches has the same type in each, and is not that of an element outside
  the choice.
- A choice without a selector is still a union (all branches overlap).
- `name` names the choice (default `body`); its type is named after what
  holds it: `SrvClanCommandBody` here, `<Type>Body` in a `complexType`.
- Rust: an enum with a variant per branch (`Created { clan, name }`,
  `NameTaken`, `Case16` for an integer case, `Other { result, .. }` for the
  default branch), as the field `body`. The selector is no field: the variant
  decides the value written for it (`body.selector()`). `Other` keeps the
  value it was read with; given the value of a named branch, it is read back
  as that branch.
- C++: the selector stays a member, the elements of all branches are members
  of what holds the choice, and reading and writing switch on the selector,
  so the bytes always follow it. The constructor stops reading at a value no
  branch is for; `pack` refuses to write one.
