//! Rust names for the things the XML defines.

/// The low `width` bits set.
pub(super) fn mask(width: u32) -> u64 {
    if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    }
}

/// `serversInfo` -> `servers_info`, `maxHP` -> `max_hp`.
pub(super) fn snake(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            let prev = chars[i - 1];
            let next_lower = chars.get(i + 1).map_or(false, |n| n.is_ascii_lowercase());
            if prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next_lower)
            {
                out.push('_');
            }
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "async", "await", "box", "final", "macro", "override", "priv", "try",
    "typeof", "virtual", "yield", "abstract", "become", "gen", "do", "unsized",
];

/// Name of a struct field or accessor: snake case, with a trailing `_` when
/// that would be a keyword.
pub(super) fn field_name(name: &str) -> String {
    let name = snake(name);
    if KEYWORDS.contains(&name.as_str()) {
        format!("{}_", name)
    } else {
        name
    }
}

/// Name of the constant for an enumerator: upper snake case.
pub(super) fn const_name(enumerator: &str) -> String {
    let name = snake(enumerator).to_ascii_uppercase();
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{}", name)
    } else {
        name
    }
}

/// Fails unless `name` can be used as a Rust identifier as it is.
pub(super) fn check_ident(name: &str, context: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(format!("{}: {:?} is not usable as a Rust identifier", context, name))
    }
}

/// A number as the XML writes it: decimal or `0x` hexadecimal.
pub(super) fn parse_int(text: &str) -> Option<u64> {
    let text = text.trim();
    match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => text.parse().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_rust_conventions() {
        assert_eq!(snake("serversInfo"), "servers_info");
        assert_eq!(snake("maxHP"), "max_hp");
        assert_eq!(snake("HPRegen"), "hp_regen");
        assert_eq!(snake("slot2Item"), "slot2_item");
        assert_eq!(field_name("type"), "type_");
        assert_eq!(field_name("ref"), "ref_");
        assert_eq!(const_name("maxBulletTypes"), "MAX_BULLET_TYPES");
        assert_eq!(const_name("2HSword"), "_2_H_SWORD");
        assert_eq!(mask(3), 0b111);
        assert_eq!(mask(64), u64::MAX);
        assert_eq!(parse_int("0x1F"), Some(31));
        assert_eq!(parse_int(" 12 "), Some(12));
        assert!(check_ident("Bad-Name", "x").is_err());
    }
}
