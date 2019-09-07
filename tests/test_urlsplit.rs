use std::env;
use std::path::PathBuf;
use std::process;

#[test]
fn test_command() {
    let workdir = get_workdir();
    let thisdir = env::current_dir().expect("Working directory");

    let output = process::Command::new(workdir.join("urlsplit"))
        .arg(thisdir.join("tests").join("in.csv"))
        .arg("-q")
        .output()
        .expect("Failed to execute urlsplit");
    let expected = include_str!("out.csv");

    let stdout = String::from_utf8(output.stdout).expect("Valid utf-8 output from urlsplit");
    let stderr = String::from_utf8(output.stderr).expect("Valid utf-8 output from urlsplit");
    assert_eq!(stderr, "");
    assert_eq!(stdout, expected);
    assert!(output.status.success());
}

fn get_workdir() -> PathBuf {
    let mut root = env::current_exe()
        .unwrap()
        .parent()
        .expect("executable's directory")
        .to_path_buf();
    if root.ends_with("deps") {
        root.pop();
    }
    root
}
