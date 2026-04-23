use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_filefile");

#[test]
fn generate_emits_mapping_form_yaml_for_a_tree() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    // Build a small tree:
    //   root/
    //     a/
    //       b  (file)
    //     c    (file)
    fs::create_dir(root.join("a")).unwrap();
    fs::write(root.join("a/b"), "").unwrap();
    fs::write(root.join("c"), "").unwrap();

    let output = Command::new(BIN)
        .args(["generate", "-p", root.to_str().unwrap(), "-s"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let yaml = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("stdout is valid yaml");

    // Top-level is a mapping.
    let map = parsed.as_mapping().expect("top-level is a mapping");
    // Must contain 'a' and 'c' keys.
    let a = map.get(&serde_yaml::Value::String("a".into())).expect("has key a");
    let c = map.get(&serde_yaml::Value::String("c".into())).expect("has key c");
    // Dir 'a' is a mapping containing 'b' as null (file).
    let a_map = a.as_mapping().expect("'a' is a mapping (dir)");
    assert!(a_map.get(&serde_yaml::Value::String("b".into())).unwrap().is_null());
    // File 'c' is null.
    assert!(c.is_null());
}
