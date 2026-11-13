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
