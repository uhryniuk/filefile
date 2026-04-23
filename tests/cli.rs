use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_filefile");

#[test]
fn positional_file_arg_applies_into_cwd() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let filefile = root.join("Filefile.yaml");
    fs::write(
        &filefile,
        "greet:\n  hello: \"world\"\n",
    )
    .unwrap();

    let status = Command::new(BIN)
        .current_dir(root)
        .arg(filefile.to_str().unwrap())
        .status()
        .unwrap();
    assert!(status.success(), "ff <file> exited non-zero");

    assert!(root.join("greet").is_dir());
    assert_eq!(fs::read_to_string(root.join("greet/hello")).unwrap(), "world");
}

#[test]
fn apply_subcommand_still_works() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let filefile = root.join("Filefile.yaml");
    fs::write(&filefile, "a: \"b\"\n").unwrap();

    let status = Command::new(BIN)
        .args(["apply", "-p", root.to_str().unwrap(), "-i", filefile.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    assert_eq!(fs::read_to_string(root.join("a")).unwrap(), "b");
}
