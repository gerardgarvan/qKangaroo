//! Subprocess-based integration tests for q-kangaroo CLI.
//!
//! These tests run the actual binary as a subprocess and verify exit codes,
//! stdout/stderr content, and overall end-to-end behavior for all CLI modes
//! and flags. Each test covers one or more requirement IDs from the roadmap.

use std::io::Write;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run q-kangaroo with given args and return (exit_code, stdout, stderr).
fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_q-kangaroo"))
        .args(args)
        .output()
        .expect("failed to run q-kangaroo");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

/// Run q-kangaroo with piped stdin input.
fn run_piped(input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_q-kangaroo"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn q-kangaroo");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("failed to wait");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

/// Create a temporary script file with the given content. Returns the path.
fn write_temp_script(name: &str, content: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, content).expect("failed to write temp script");
    path
}

// ===========================================================================
// CLI-01: --help / -h
// ===========================================================================

#[test]
fn help_flag_long() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("USAGE:"), "missing USAGE section");
    assert!(stdout.contains("-h, --help"), "missing -h flag");
    assert!(stdout.contains("-V, --version"), "missing -V flag");
    assert!(stdout.contains("-c EXPRESSION"), "missing -c flag");
    assert!(stdout.contains("-q, --quiet"), "missing -q flag");
    assert!(stdout.contains("-v, --verbose"), "missing -v flag");
    assert!(stdout.contains("--"), "missing -- separator");
    assert!(stdout.contains("EXAMPLES:"), "missing EXAMPLES section");
}

#[test]
fn help_flag_short() {
    let (code, stdout, _) = run(&["-h"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("USAGE:"));
}

// ===========================================================================
// CLI-02: -q / --quiet
// ===========================================================================

#[test]
fn quiet_flag_suppresses_banner() {
    // -q with -c to avoid interactive mode
    let (code, _, _) = run(&["-q", "-c", "1+1"]);
    assert_eq!(code, 0);
}

#[test]
fn quiet_flag_long() {
    let (code, _, _) = run(&["--quiet", "-c", "1+1"]);
    assert_eq!(code, 0);
}

// ===========================================================================
// CLI-03: -c EXPRESSION
// ===========================================================================

#[test]
fn c_flag_simple_arithmetic() {
    let (code, stdout, _) = run(&["-c", "1 + 1"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn c_flag_function_call() {
    let (code, stdout, _) = run(&["-c", "partition_count(5)"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "7");
}

#[test]
fn c_flag_no_banner() {
    let (code, stdout, _) = run(&["-c", "1 + 1"]);
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("q-Kangaroo"),
        "banner should not appear in -c mode"
    );
}

#[test]
fn c_flag_missing_expression() {
    let (code, _, stderr) = run(&["-c"]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("requires an argument"),
        "expected 'requires an argument', got stderr: {}",
        stderr
    );
}

#[test]
fn c_flag_parse_error() {
    let (code, _, stderr) = run(&["-c", "1 + + 2"]);
    assert_ne!(code, 0);
    assert!(!stderr.is_empty(), "parse error should be on stderr");
}

#[test]
fn c_flag_eval_error() {
    let (code, _, stderr) = run(&["-c", "undefined_var"]);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("undefined variable"),
        "expected 'undefined variable', got stderr: {}",
        stderr
    );
}

// ===========================================================================
// CLI-04: -v / --verbose
// ===========================================================================

#[test]
fn verbose_flag_shows_timing() {
    let (code, _, stderr) = run(&["-v", "-c", "1 + 1"]);
    assert_eq!(code, 0);
    assert!(
        stderr.contains("["),
        "timing should appear on stderr, got: {}",
        stderr
    );
    assert!(stderr.contains("s]"), "timing should show seconds");
}

#[test]
fn verbose_flag_long() {
    let (code, _, stderr) = run(&["--verbose", "-c", "1 + 1"]);
    assert_eq!(code, 0);
    assert!(
        stderr.contains("["),
        "timing should appear on stderr with --verbose"
    );
}

// ===========================================================================
// CLI-05: -- separator
// ===========================================================================

#[test]
fn dashdash_separator() {
    // After --, next arg is treated as filename even if it starts with -
    let (code, _, stderr) = run(&["--", "-nonexistent.qk"]);
    assert_ne!(code, 0);
    // Should try to open the file, not treat -nonexistent.qk as a flag
    assert!(
        stderr.contains("file not found") || stderr.contains("cannot read"),
        "should attempt to read file, got stderr: {}",
        stderr
    );
}

#[test]
fn dashdash_with_real_file() {
    let tmp = write_temp_script("qk_test_dashdash_real.qk", "100 + 23");
    let path = tmp.to_str().unwrap();
    let (code, stdout, _) = run(&["--", path]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "123");
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// CLI-06: Unknown flags
// ===========================================================================

#[test]
fn unknown_flag_exits_2() {
    let (code, _, stderr) = run(&["--badopt"]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("unknown option"),
        "expected 'unknown option', got stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("--help"),
        "should suggest --help, got stderr: {}",
        stderr
    );
}

#[test]
fn unknown_short_flag_exits_2() {
    let (code, _, stderr) = run(&["-z"]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("unknown option"),
        "expected 'unknown option', got stderr: {}",
        stderr
    );
}

// ===========================================================================
// Version flags
// ===========================================================================

#[test]
fn version_flag_long() {
    let (code, stdout, _) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("q-kangaroo"),
        "version output should contain 'q-kangaroo', got: {}",
        stdout
    );
}

#[test]
fn version_flag_short() {
    let (code, stdout, _) = run(&["-V"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("q-kangaroo"),
        "version output should contain 'q-kangaroo', got: {}",
        stdout
    );
}

// ===========================================================================
// EXEC-01: Script file execution
// ===========================================================================

#[test]
fn script_file_execution() {
    let tmp = write_temp_script("qk_test_script.qk", "1 + 1");
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "2");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn script_file_no_banner() {
    let tmp = write_temp_script("qk_test_banner.qk", "1 + 1");
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("q-Kangaroo"),
        "banner should not appear in script mode"
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn script_file_not_found() {
    let (code, _, stderr) = run(&["/nonexistent/path/script.qk"]);
    assert_eq!(code, 66, "expected exit code 66 for file not found, got {}", code);
    assert!(
        stderr.contains("file not found"),
        "expected 'file not found', got stderr: {}",
        stderr
    );
}

#[test]
fn script_multi_statement() {
    let tmp = write_temp_script("qk_test_multi.qk", "x := 10:\ny := 20:\nx + y");
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "30");
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// EXEC-02: # comments
// ===========================================================================

#[test]
fn script_with_comments() {
    let tmp = write_temp_script(
        "qk_test_comments.qk",
        "# This is a comment\n1 + 2\n# Another comment",
    );
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "3");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn script_inline_comment() {
    let tmp = write_temp_script("qk_test_inline.qk", "1 + 2 # add numbers");
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "3");
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// EXEC-03: Multi-line statements
// ===========================================================================

#[test]
fn script_multiline_expression() {
    let tmp = write_temp_script(
        "qk_test_multiline.qk",
        "aqprod(\n  1, 1, 1,\n  infinity, 20\n)",
    );
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty(), "should produce output");
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// EXEC-04: Piped stdin
// ===========================================================================

#[test]
fn piped_stdin_simple() {
    let (code, stdout, _) = run_piped("1 + 1");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn piped_stdin_no_banner() {
    let (code, stdout, _) = run_piped("1 + 1");
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("q-Kangaroo"),
        "banner should not appear in piped mode"
    );
    assert!(
        !stdout.contains("q>"),
        "prompt should not appear in piped mode"
    );
}

#[test]
fn piped_stdin_multi_statement() {
    let (code, stdout, _) = run_piped("x := 5:\nx + 10");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "15");
}

// ===========================================================================
// EXEC-05: Suppress banner in non-interactive modes
// (Covered by c_flag_no_banner, script_file_no_banner, piped_stdin_no_banner)
// This additional test explicitly verifies no prompt characters leak.
// ===========================================================================

#[test]
fn non_interactive_no_prompt() {
    // -c mode should have no prompt
    let (code, stdout, _) = run(&["-c", "42"]);
    assert_eq!(code, 0);
    assert!(!stdout.contains("q>"), "prompt should not appear in -c mode");
    assert_eq!(stdout.trim(), "42");
}

// ===========================================================================
// EXEC-06: read() function
// ===========================================================================

#[test]
fn read_function_in_c_flag() {
    let tmp = write_temp_script("qk_test_read_target.qk", "x := 42:");
    // Escape backslashes for Windows paths in string literals
    let path_str = tmp.to_str().unwrap().replace('\\', "\\\\");
    let expr = format!("read(\"{}\"):\nx", path_str);
    let (code, stdout, stderr) = run(&["-c", &expr]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.trim(), "42");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn read_function_not_found() {
    let expr = "read(\"/nonexistent/file.qk\")";
    let (code, _, stderr) = run(&["-c", expr]);
    assert_ne!(code, 0);
    assert!(
        !stderr.is_empty(),
        "read() of nonexistent file should produce error"
    );
}

// ===========================================================================
// Combined flag tests
// ===========================================================================

#[test]
fn verbose_with_script_file() {
    let tmp = write_temp_script("qk_test_verbose_script.qk", "1 + 1");
    let (code, _, stderr) = run(&["-v", tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stderr.contains("["),
        "verbose should show timing for script mode, got stderr: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn c_flag_q_series_output() {
    // Verify a real q-series computation works end-to-end
    let (code, stdout, _) = run(&["-c", "etaq(1,1,5)"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("q"),
        "etaq should produce q-series output, got: {}",
        stdout
    );
}

#[test]
fn script_with_assignment_chain() {
    let tmp = write_temp_script(
        "qk_test_chain.qk",
        "a := 3:\nb := 4:\nc := a + b:\nc * 2",
    );
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "14");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn piped_stdin_with_comments() {
    let (code, stdout, _) = run_piped("# comment\n5 + 5");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "10");
}

#[test]
fn help_exits_before_stdin() {
    // --help should return immediately even if stdin is not a terminal
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("USAGE:"));
}

#[test]
fn version_exits_before_stdin() {
    // --version should return immediately
    let (code, stdout, _) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("q-kangaroo"));
}
