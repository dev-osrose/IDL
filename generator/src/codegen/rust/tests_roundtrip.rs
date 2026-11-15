//! Compiles and runs the generated code of the selector choice packets
//! against a small runtime (`roundtrip/`): a string match cannot show that a
//! packet reads back what it wrote.

use std::path::PathBuf;
use std::process::Command;

use super::tests::generate;
use super::tests_selected::{CLAN, ENUM_DEFAULT, EVENT, IN_A_TYPE};

#[test]
fn generated_selector_choices_round_trip() {
    let dir = std::env::temp_dir().join(format!("idl-selector-roundtrip-{}", std::process::id()));
    let src = dir.join("src");
    let packets = src.join("packets");
    std::fs::create_dir_all(&packets).unwrap();
    for (name, content) in generate(&[CLAN, EVENT, IN_A_TYPE, ENUM_DEFAULT]) {
        std::fs::write(packets.join(name), content).unwrap();
    }
    let fixture = |name: &str| -> String {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src/codegen/rust/roundtrip", name].iter().collect();
        std::fs::read_to_string(path).unwrap()
    };
    std::fs::write(src.join("lib.rs"), fixture("lib.rs") + "#[cfg(test)]\nmod cases;\n").unwrap();
    for name in ["runtime.rs", "testgen.rs", "cases.rs"] {
        let to = if name == "runtime.rs" { "wire.rs" } else { name };
        std::fs::write(src.join(to), fixture(name)).unwrap();
    }
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"roundtrip\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[features]\ndefault = [\"testgen\"]\ntestgen = []\n\n[workspace]\n",
    )
    .unwrap();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = Command::new(cargo)
        .args(["test", "--offline", "--quiet"])
        .current_dir(&dir)
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .env_remove("RUSTFLAGS")
        .output()
        .unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "generated code failed in {}:\n{}", dir.display(), text);
    assert!(text.contains("test result: ok. 8 passed"), "not every case ran:\n{}", text);
    let _ = std::fs::remove_dir_all(&dir);
}
