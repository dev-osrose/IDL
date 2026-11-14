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
