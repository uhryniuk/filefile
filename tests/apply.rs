use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_filefile");

fn apply(root: &std::path::Path, filefile: &std::path::Path) {
    let status = Command::new(BIN)
        .args(["apply", "-p", root.to_str().unwrap(), "-i", filefile.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "apply exited non-zero");
}

#[test]
fn apply_creates_nested_dir_and_file_with_contents() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let filefile = root.join("Filefile.yaml");
    fs::write(
        &filefile,
        "hello:\n  world: \"contents of the file\"\n  here:\n    I: \"am\"\n",
    )
    .unwrap();

    apply(root, &filefile);

    assert!(root.join("hello").is_dir());
    assert_eq!(
        fs::read_to_string(root.join("hello/world")).unwrap(),
        "contents of the file"
    );
    assert!(root.join("hello/here").is_dir());
    assert_eq!(fs::read_to_string(root.join("hello/here/I")).unwrap(), "am");
}

#[test]
fn sh_op_captures_stdout_into_node_file() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let filefile = root.join("Filefile.yaml");
    fs::write(&filefile, "marker: !sh \"printf hi\"\n").unwrap();

    apply(root, &filefile);

    assert_eq!(fs::read_to_string(root.join("marker")).unwrap(), "hi");
}
