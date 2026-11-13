//! The C++ types the XML uses directly, and how they map to Rust.

pub(super) struct Prim {
    pub rust: &'static str,
    pub bits: u32,
    /// Usable as a count, a bit field or the storage of a union.
    pub integer: bool,
    pub unsigned: bool,
}

pub(super) fn prim(ty: &str) -> Option<Prim> {
    let p = |rust, bits, integer, unsigned| {
        Some(Prim {
            rust,
            bits,
            integer,
            unsigned,
        })
    };
    match ty.trim() {
        "uint8_t" | "char" => p("u8", 8, true, true),
        "int8_t" => p("i8", 8, true, false),
        "uint16_t" => p("u16", 16, true, true),
        "int16_t" => p("i16", 16, true, false),
        "uint32_t" | "unsigned int" => p("u32", 32, true, true),
        "int32_t" | "int" => p("i32", 32, true, false),
        "uint64_t" => p("u64", 64, true, true),
        "int64_t" => p("i64", 64, true, false),
        "float" => p("f32", 32, false, false),
        "double" => p("f64", 64, false, false),
        "std::string" => p("BString", 0, false, false),
        _ => None,
    }
}
