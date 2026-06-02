#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Cursor;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use zip::ZipWriter;

const BIN: &str = env!("CARGO_BIN_EXE_zipcrawl");

fn create_zip(contents: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);
    for (name, content) in contents {
        zip.start_file::<&str, ()>(name, Default::default())
            .unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    buf.into_inner()
}

fn make_zip_file(contents: &[(&str, &str)]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.zip");
    let data = create_zip(contents);
    std::fs::write(&path, data).unwrap();
    (dir, path)
}

fn exec(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(BIN)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn exec_zip(zip_path: &Path, args: &[&str]) -> (bool, String, String) {
    let mut all_args = vec![zip_path.to_str().unwrap()];
    all_args.extend_from_slice(args);
    exec(&all_args)
}

#[test]
fn tree_shows_structure() {
    let (_dir, zip) = make_zip_file(&[("dir/file.txt", "hello")]);
    let (ok, stdout, _) = exec_zip(&zip, &["tree"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("file.txt"));
}

#[test]
fn tree_respects_depth() {
    let (_dir, zip) = make_zip_file(&[("a/b/c/d/file.txt", "deep")]);
    let (ok, stdout, _) = exec_zip(&zip, &["tree", "--depth", "2"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("b"), "depth 2 should show up to b");
    assert!(!stdout.contains("c"), "depth 2 should not show c");
}

#[test]
fn list_shows_entries() {
    let (_dir, zip) = make_zip_file(&[("a.txt", "aaa"), ("b.txt", "bbb")]);
    let (ok, stdout, _) = exec_zip(&zip, &["list"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("b.txt"));
}

#[test]
fn list_with_sizes() {
    let (_dir, zip) = make_zip_file(&[("data.txt", "hello world")]);
    let (ok, stdout, _) = exec_zip(&zip, &["list", "--sizes"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("data.txt"));
    assert!(stdout.contains("B"));
}

#[test]
fn cat_shows_content() {
    let (_dir, zip) = make_zip_file(&[("hello.txt", "Hello, World!")]);
    let (ok, stdout, _) = exec_zip(&zip, &["cat", "hello.txt"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("Hello, World!"));
}

#[test]
fn cat_with_glob() {
    let (_dir, zip) = make_zip_file(&[("a.rs", "fn a() {}"), ("b.py", "def b(): pass")]);
    let (ok, stdout, _) = exec_zip(&zip, &["cat", "*.rs"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("fn a()"));
    assert!(!stdout.contains("def b()"), "should not match .py files");
}

#[test]
fn cat_quiet_shows_no_header() {
    let (_dir, zip) = make_zip_file(&[("hello.txt", "content")]);
    let (ok, stdout, _) = exec_zip(&zip, &["cat", "hello.txt", "--quiet"]);
    assert!(ok, "stdout: {stdout}");
    assert_eq!(stdout.trim(), "content");
}

#[test]
fn cat_file_not_found() {
    let (_dir, zip) = make_zip_file(&[("exists.txt", "content")]);
    let (ok, _, _) = exec_zip(&zip, &["cat", "missing.txt"]);
    assert!(!ok, "should fail for missing file");
}

#[test]
fn find_by_regex() {
    let (_dir, zip) = make_zip_file(&[("main.rs", ""), ("lib.rs", ""), ("readme.md", "")]);
    let (ok, stdout, _) = exec_zip(&zip, &["find", r"\.rs$"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("main.rs"));
    assert!(stdout.contains("lib.rs"));
    assert!(!stdout.contains("readme.md"));
}

#[test]
fn find_by_glob() {
    let (_dir, zip) = make_zip_file(&[("data.json", "{}"), ("data.yaml", "{}")]);
    let (ok, stdout, _) = exec_zip(&zip, &["find", "*.json", "--glob"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("data.json"));
    assert!(!stdout.contains("data.yaml"));
}

#[test]
fn find_filter_by_type() {
    let (_dir, zip) = make_zip_file(&[("file.txt", ""), ("dir/", "")]);
    let (ok, stdout, _) = exec_zip(&zip, &["find", ".", "--entry-type", "d"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("dir/") || stdout.contains("dir"));
}

#[test]
fn find_invalid_regex_errors() {
    let (_dir, zip) = make_zip_file(&[("file.txt", "")]);
    let (ok, _, stderr) = exec_zip(&zip, &["find", r"["]);
    assert!(!ok, "should fail for invalid regex");
    assert!(
        stderr.contains("regex") || stderr.contains("invalid"),
        "stderr: {stderr}"
    );
}

#[test]
fn find_invalid_glob_errors() {
    let (_dir, zip) = make_zip_file(&[("file.txt", "")]);
    let (ok, _, _stderr) = exec_zip(&zip, &["find", r"[", "--glob"]);
    assert!(!ok, "should fail for invalid glob");
}

#[test]
fn grep_finds_pattern() {
    let (_dir, zip) = make_zip_file(&[("code.rs", "fn hello() {\n  println!(\"hi\");\n}")]);
    let (ok, stdout, _) = exec_zip(&zip, &["grep", "hello"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("hello"));
}

#[test]
fn grep_no_match() {
    let (_dir, zip) = make_zip_file(&[("data.txt", "some content")]);
    let (ok, stdout, _) = exec_zip(&zip, &["grep", "nonexistent"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.is_empty());
}

#[test]
fn grep_invalid_regex_errors() {
    let (_dir, zip) = make_zip_file(&[("file.txt", "")]);
    let (ok, _, stderr) = exec_zip(&zip, &["grep", r"["]);
    assert!(!ok, "should fail for invalid regex");
    assert!(
        stderr.contains("regex") || stderr.contains("invalid"),
        "stderr: {stderr}"
    );
}

#[test]
fn grep_with_glob_filter() {
    let (_dir, zip) = make_zip_file(&[("a.rs", "fn foo() {}"), ("b.txt", "foo")]);
    let (ok, stdout, _) = exec_zip(&zip, &["grep", "foo", "--glob", "*.rs"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("a.rs"));
    assert!(!stdout.contains("b.txt"));
}

#[test]
fn execute_runs_command() {
    let (_dir, zip) = make_zip_file(&[("data.txt", "hello world")]);
    let (ok, stdout, _) = exec_zip(&zip, &["x", "data.txt", "cat"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("hello world"));
}

#[test]
fn invalid_zip_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.zip");
    std::fs::write(&path, b"not a zip file").unwrap();
    let (ok, _, _) = exec_zip(&path, &["list"]);
    assert!(!ok);
}

#[test]
fn multiple_archives() {
    let (_d1, z1) = make_zip_file(&[("a.txt", "aaa")]);
    let (_d2, z2) = make_zip_file(&[("b.txt", "bbb")]);
    let (ok, stdout, _) = exec(&[z1.to_str().unwrap(), z2.to_str().unwrap(), "list"]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("b.txt"));
}

#[test]
fn diff_default_shows_additions() {
    let (_d1, base) = make_zip_file(&[("common.txt", "base")]);
    let (_d2, current) = make_zip_file(&[("common.txt", "base"), ("new.txt", "new")]);
    let (ok, stdout, _) = exec_zip(&current, &["diff", "--base", base.to_str().unwrap()]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("new.txt"), "should show added file");
}

#[test]
fn diff_default_shows_removals() {
    let (_d1, base) = make_zip_file(&[("common.txt", "base"), ("old.txt", "old")]);
    let (_d2, current) = make_zip_file(&[("common.txt", "base")]);
    let (ok, stdout, _) = exec_zip(&current, &["diff", "--base", base.to_str().unwrap()]);
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("old.txt"), "should show removed file");
}

#[test]
fn diff_stats_shows_sizes() {
    let (_d1, base) = make_zip_file(&[("data.txt", "short")]);
    let (_d2, current) = make_zip_file(&[("data.txt", "a longer version of the file content")]);
    let (ok, stdout, _) = exec_zip(
        &current,
        &["diff", "--base", base.to_str().unwrap(), "--mode", "stats"],
    );
    assert!(ok, "stdout: {stdout}");
    assert!(
        stdout.contains("size"),
        "stats mode should show size changes"
    );
}

#[test]
fn diff_structure_only() {
    let (_d1, base) = make_zip_file(&[("keep.txt", "")]);
    let (_d2, current) = make_zip_file(&[("keep.txt", ""), ("extra.txt", "")]);
    let (ok, stdout, _) = exec_zip(
        &current,
        &[
            "diff",
            "--base",
            base.to_str().unwrap(),
            "--mode",
            "structure",
        ],
    );
    assert!(ok, "stdout: {stdout}");
    assert!(stdout.contains("extra.txt"));
    assert!(
        !stdout.contains("size"),
        "structure mode should not show sizes"
    );
}
