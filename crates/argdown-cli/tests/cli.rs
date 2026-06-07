use std::io::Write;
use std::process::{Command, Stdio};

/// Run the `argdown` binary with `args`, feeding `stdin`. Returns
/// `(stdout, stderr, exit_code)`.
fn run(args: &[&str], stdin: &str) -> (String, String, Option<i32>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_argdown"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn argdown");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait for argdown");
    (
        String::from_utf8(out.stdout).expect("utf8 stdout"),
        String::from_utf8(out.stderr).expect("utf8 stderr"),
        out.status.code(),
    )
}

#[test]
fn parse_valid_prints_summary_json_to_stdout() {
    let (stdout, stderr, code) = run(&["parse"], "<A>: a");
    assert_eq!(code, Some(0));
    assert!(stderr.is_empty(), "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is JSON");
    assert_eq!(v["arguments"], 1);
}

#[test]
fn parse_malformed_writes_diagnostic_to_stderr_and_exits_one() {
    let (stdout, stderr, code) = run(&["parse"], "# H {unterminated");
    assert_eq!(code, Some(1));
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.starts_with("argdown:"), "stderr: {stderr}");
}
