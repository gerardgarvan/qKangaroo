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
    // Use a real eval error: wrong argument count for etaq
    let (code, _, stderr) = run(&["-c", "etaq(1)"]);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("expects"),
        "expected arg count error, got stderr: {}",
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

// ===========================================================================
// EXIT-01: Exit code 0 on success (explicit requirement label)
// ===========================================================================

#[test]
fn exit_01_success_exit_code() {
    let (code, stdout, _) = run(&["-c", "1 + 1"]);
    assert_eq!(code, 0, "EXIT-01: success should exit 0");
    assert_eq!(stdout.trim(), "2");
}

// ===========================================================================
// EXIT-02: Exit code 1 on evaluation error in batch mode
// ===========================================================================

#[test]
fn exit_02_eval_error_exit_code() {
    // Use a real eval error: wrong argument count
    let (code, _, stderr) = run(&["-c", "etaq(1)"]);
    assert_eq!(code, 1, "EXIT-02: eval error should exit 1");
    assert!(stderr.contains("expects"), "expected arg count error, got: {}", stderr);
}

#[test]
fn exit_02_eval_error_in_script() {
    // Use a real eval error: wrong argument count for etaq
    let tmp = write_temp_script("qk_test_exit02.qk", "x := 1:\netaq(1)");
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 1, "EXIT-02: script eval error should exit 1");
    assert!(stderr.contains("expects"), "expected arg count error, got: {}", stderr);
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// EXIT-03: Exit code 2 on usage error (explicit label)
// ===========================================================================

#[test]
fn exit_03_usage_error_exit_code() {
    let (code, _, stderr) = run(&["--bogus"]);
    assert_eq!(code, 2, "EXIT-03: unknown flag should exit 2");
    assert!(stderr.contains("unknown option"));
    assert!(stderr.contains("--help"), "should suggest --help");
}

// ===========================================================================
// EXIT-04: Exit code 65 on parse error in script input
// ===========================================================================

#[test]
fn exit_04_parse_error_exit_code() {
    let (code, _, stderr) = run(&["-c", "1 + + 2"]);
    assert_eq!(code, 65, "EXIT-04: parse error should exit 65");
    assert!(stderr.contains("parse error"));
}

#[test]
fn exit_04_parse_error_in_script() {
    let tmp = write_temp_script("qk_test_exit04.qk", "1 + + 2");
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 65, "EXIT-04: script parse error should exit 65");
    assert!(stderr.contains("parse error"));
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// EXIT-05: Exit code 66 on file not found
// ===========================================================================

#[test]
fn exit_05_file_not_found_exit_code() {
    let (code, _, stderr) = run(&["nonexistent_script_exit05.qk"]);
    assert_eq!(code, 66, "EXIT-05: file not found should exit 66");
    assert!(
        stderr.contains("file not found"),
        "EXIT-05: stderr should contain 'file not found', got: {}",
        stderr
    );
    // ERR-03: Should include OS error message
    // Windows: "The system cannot find the file specified" or similar
    // Unix: "No such file or directory"
    assert!(
        stderr.contains("os error") || stderr.contains("No such file"),
        "EXIT-05/ERR-03: should include OS error message, got: {}",
        stderr
    );
}

// ===========================================================================
// EXIT-06: Exit code 70 on caught panic
// ===========================================================================

#[test]
fn exit_06_panic_invert_zero_constant() {
    // etaq(1,1,5) - 1 has zero constant term, so inverting it panics
    let tmp = write_temp_script("qk_test_exit06.qk", "1/(etaq(1,1,5) - 1)");
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 70, "EXIT-06: caught panic should exit 70");
    // ERR-02: Should show translated message, not raw assert text.
    // The panic hook suppresses the raw "thread 'main' panicked at ..." output.
    assert!(
        !stderr.contains("Cannot invert series with zero constant term"),
        "ERR-02: should show translated message, not raw panic. Got: {}",
        stderr
    );
    assert!(
        stderr.contains("cannot invert") || stderr.contains("constant term is zero"),
        "ERR-02: should show friendly version of the panic. Got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn exit_06_division_by_zero_panic() {
    let (code, _, stderr) = run(&["-c", "1/0"]);
    assert_eq!(code, 70, "EXIT-06: division by zero panic should exit 70");
    assert!(
        stderr.contains("division by zero"),
        "should contain 'division by zero', got: {}",
        stderr
    );
    // Should NOT contain raw Rust panic prefix
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "ERR-02: raw panic output should be suppressed, got: {}",
        stderr
    );
}

// ===========================================================================
// EXIT-07: Exit code 74 on I/O error
// ===========================================================================

#[test]
fn exit_07_io_error_directory_as_file() {
    // Reading a directory as a file produces an I/O error (not NotFound)
    // On Windows this gives "Access is denied" with exit 74
    let (code, _, stderr) = run(&["."]);
    assert!(
        code == 74 || code == 66 || code == 1,
        "EXIT-07: I/O error reading directory should exit 74 (or 66 on some platforms), got: {}",
        code
    );
    assert!(!stderr.is_empty(), "should produce an error message");
}

// ===========================================================================
// ERR-01: Script errors include filename:line:col context
// ===========================================================================

#[test]
fn err_01_parse_error_shows_filename_line_col() {
    let tmp = write_temp_script(
        "qk_test_err01_parse.qk",
        "x := 1:\ny := 2:\n1 + + 3",
    );
    let path_str = tmp.to_str().unwrap();
    let (code, _, stderr) = run(&[path_str]);
    assert_eq!(code, 65);
    // Should contain filename:line:col format
    assert!(
        stderr.contains(":3:"),
        "ERR-01: parse error on line 3 should show ':3:' in error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("parse error"),
        "ERR-01: should contain 'parse error', got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn err_01_eval_error_shows_filename_line() {
    // Use a real eval error on line 5: wrong argument count
    let tmp = write_temp_script(
        "qk_test_err01_eval.qk",
        "x := 1:\ny := 2:\nz := 3:\nw := 4:\netaq(1)",
    );
    let path_str = tmp.to_str().unwrap();
    let (code, _, stderr) = run(&[path_str]);
    assert_eq!(code, 1);
    // Should contain filename:line format (line 5 has the error)
    assert!(
        stderr.contains(":5:"),
        "ERR-01: eval error on line 5 should show ':5:' in error, got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn err_01_first_line_error() {
    // Use a real eval error on line 1
    let tmp = write_temp_script("qk_test_err01_first.qk", "etaq(1)");
    let path_str = tmp.to_str().unwrap();
    let (code, _, stderr) = run(&[path_str]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains(":1:"),
        "ERR-01: error on line 1 should show ':1:', got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// ERR-02: Panic messages translated to human-readable
// (Also covered by exit_06_panic_invert_zero_constant above)
// ===========================================================================

#[test]
fn err_02_division_by_zero_translated() {
    let tmp = write_temp_script("qk_test_err02_div.qk", "1/0");
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 70, "division by zero should exit 70");
    // Should say "division by zero" (translated, not "QRat division by zero")
    assert!(
        stderr.contains("division by zero"),
        "ERR-02: should contain 'division by zero', got: {}",
        stderr
    );
    // Should NOT contain internal type prefixes
    assert!(
        !stderr.contains("QRat"),
        "ERR-02: should not contain internal type name 'QRat', got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// ERR-03: File I/O errors display OS error message
// (Also covered by exit_05_file_not_found_exit_code above)
// ===========================================================================

#[test]
fn err_03_file_error_includes_os_message() {
    let (code, _, stderr) = run(&["nonexistent_file_abc123.qk"]);
    assert_eq!(code, 66);
    // OS error message should be included (Windows says "The system cannot find
    // the file specified" / Unix says "No such file or directory")
    assert!(
        stderr.len() > 30,
        "ERR-03: error should include OS message, got short stderr: {}",
        stderr
    );
    // Should contain the OS error indicator
    assert!(
        stderr.contains("os error") || stderr.contains("No such file"),
        "ERR-03: should include OS-level error detail, got: {}",
        stderr
    );
}

// ===========================================================================
// ERR-04: Scripts fail-fast on first error; REPL continues
// ===========================================================================

#[test]
fn err_04_script_fail_fast() {
    // Two errors in script separated by terminators -- only the first should appear.
    // Use real eval errors: wrong argument count for different functions.
    let tmp = write_temp_script(
        "qk_test_err04.qk",
        "x := 1:\netaq(1);\naqprod(1)",
    );
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_ne!(code, 0, "should fail on first error");
    // Should mention etaq but NOT aqprod (fail-fast on first error)
    assert!(
        stderr.contains("etaq"),
        "ERR-04: should report first error (etaq), got: {}",
        stderr
    );
    assert!(
        !stderr.contains("aqprod"),
        "ERR-04: should NOT report second error (aqprod) due to fail-fast, got: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// ERR-05: read() in REPL continues on error; read() error message quality
// ===========================================================================

#[test]
fn err_05_read_nonexistent_shows_file_not_found() {
    let expr = r#"read("/nonexistent/file.qk")"#;
    let (code, _, stderr) = run(&["-c", expr]);
    assert_ne!(code, 0);
    // Should show "file not found" not "computation failed"
    assert!(
        stderr.contains("file not found"),
        "ERR-05: read() of nonexistent file should show 'file not found', got: {}",
        stderr
    );
}

#[test]
fn err_05_read_error_not_computation_failed() {
    // read() file errors should NOT say "computation failed"
    let expr = r#"read("/no/such/file.qk")"#;
    let (_, _, stderr) = run(&["-c", expr]);
    assert!(
        !stderr.contains("computation failed"),
        "ERR-05: read() file error should NOT say 'computation failed', got: {}",
        stderr
    );
}

// ===========================================================================
// SYM-01: Bare symbols (Phase 33)
// ===========================================================================

#[test]
fn symbol_bare_variable() {
    // Typing an undefined name returns the symbol itself
    let (code, stdout, _) = run(&["-c", "f"]);
    assert_eq!(code, 0, "bare symbol should succeed");
    assert_eq!(stdout.trim(), "f");
}

#[test]
fn symbol_q_bare() {
    // q is now a regular symbol (no longer a keyword)
    let (code, stdout, _) = run(&["-c", "q"]);
    assert_eq!(code, 0, "bare q should succeed as symbol");
    assert_eq!(stdout.trim(), "q");
}

// ===========================================================================
// SYM-02: etaq(q, 1, 20) works (Phase 33)
// ===========================================================================

#[test]
fn sym_02_etaq_with_q_symbol() {
    let (code, stdout, stderr) = run(&["-c", "etaq(q, 1, 20)"]);
    assert_eq!(code, 0, "SYM-02: etaq(q, 1, 20) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q"), "should contain q variable");
    assert!(stdout.contains("O(q^20)"), "should show truncation at order 20");
}

#[test]
fn sym_02_etaq_with_t_symbol() {
    // Any symbol can be the base variable
    let (code, stdout, stderr) = run(&["-c", "etaq(t, 1, 10)"]);
    assert_eq!(code, 0, "SYM-02: etaq(t, 1, 10) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("t"), "should display in variable t");
    assert!(stdout.contains("O(t^10)"), "should show truncation with t variable");
}

#[test]
fn sym_02_etaq_legacy_still_works() {
    // Legacy syntax etaq(b, t, order) should still work
    let (code, stdout, _) = run(&["-c", "etaq(1, 1, 10)"]);
    assert_eq!(code, 0, "legacy etaq(1,1,10) should still work");
    assert!(stdout.contains("q"), "legacy output uses q");
}

// ===========================================================================
// SYM-03: aqprod(q^2, q, 5) works (Phase 33)
// ===========================================================================

#[test]
fn sym_03_aqprod_with_monomial() {
    let (code, stdout, stderr) = run(&["-c", "aqprod(q^2, q, 5)"]);
    assert_eq!(code, 0, "SYM-03: aqprod(q^2, q, 5) should succeed. stderr: {}", stderr);
    // Should produce a series/polynomial
    assert!(!stdout.trim().is_empty(), "should produce output");
}

#[test]
fn sym_03_aqprod_legacy_still_works() {
    // Legacy: aqprod(1, 1, 1, infinity, 20)
    let (code, stdout, _) = run(&["-c", "aqprod(1, 1, 1, infinity, 20)"]);
    assert_eq!(code, 0, "legacy aqprod should still work");
    assert!(stdout.contains("q"), "should produce q-series");
}

// ===========================================================================
// SYM-04: Assignment precedence (Phase 33)
// ===========================================================================

#[test]
fn symbol_assignment_precedence() {
    // Assigned variables return their value, not a symbol
    let (code, stdout, _) = run(&["-c", "x := 42:\nx"]);
    assert_eq!(code, 0, "assigned variable should succeed");
    assert_eq!(stdout.trim(), "42");
}

#[test]
fn sym_04_q_reassignment() {
    // q is NOT protected from reassignment
    let (code, stdout, _) = run(&["-c", "q := 5:\nq"]);
    assert_eq!(code, 0, "q should be reassignable");
    assert_eq!(stdout.trim(), "5", "q should return assigned value");
}

// ===========================================================================
// Polynomial arithmetic and display (Phase 33)
// ===========================================================================

#[test]
fn polynomial_arithmetic() {
    let (code, stdout, _) = run(&["-c", "(q + 1) * (q + 1)"]);
    assert_eq!(code, 0, "polynomial multiplication should work");
    // Should be 1 + 2*q + q^2 with no O(...)
    assert!(stdout.contains("1"), "should have constant term");
    assert!(stdout.contains("q^2"), "should have q^2 term");
    assert!(!stdout.contains("O("), "polynomial should not have O(...) truncation");
}

#[test]
fn polynomial_display_no_truncation() {
    let (code, stdout, _) = run(&["-c", "q^2 + q + 1"]);
    assert_eq!(code, 0);
    assert!(!stdout.contains("O("), "polynomial should not have O(...) truncation");
}

#[test]
fn series_display_has_truncation() {
    let (code, stdout, _) = run(&["-c", "etaq(1, 1, 5)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^5)"), "series should have O(q^5) truncation");
}

// ===========================================================================
// Variable management: restart, anames, unassign (Phase 33)
// ===========================================================================

#[test]
fn restart_function_in_script() {
    // restart() clears all variables; anames() should return empty list after
    let tmp = write_temp_script("qk_test_restart.qk", "x := 42:\nrestart():\nanames()");
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "restart in script should work. stderr: {}", stderr);
    // After restart(), anames() should return empty list
    assert!(stdout.contains("[]"), "anames() after restart should be empty. stdout: {}", stdout);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn anames_function() {
    let (code, stdout, _) = run(&["-c", "x := 1:\ny := 2:\nanames()"]);
    assert_eq!(code, 0, "anames() should work");
    // Should list x and y
    assert!(stdout.contains("x"), "should list x");
    assert!(stdout.contains("y"), "should list y");
}

#[test]
fn anames_empty() {
    let (code, stdout, _) = run(&["-c", "anames()"]);
    assert_eq!(code, 0, "anames() with no vars should work");
    assert_eq!(stdout.trim(), "[]", "should be empty list");
}

#[test]
fn unassign_via_single_quote() {
    let tmp = write_temp_script("qk_test_unassign.qk", "x := 42:\nx := 'x':\nx");
    let (code, stdout, _) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "unassign should work");
    // After unassign, x is a symbol again
    assert_eq!(stdout.trim(), "x", "after unassign, x should be a symbol");
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// Additional Phase 33 regression: long symbol names (SYM-01)
// ===========================================================================

#[test]
fn sym_01_long_name_symbol() {
    let (code, stdout, _) = run(&["-c", "myVariable"]);
    assert_eq!(code, 0, "SYM-01: long names should work");
    assert_eq!(stdout.trim(), "myVariable");
}

// ===========================================================================
// Phase 34: Product & Theta Signatures -- Maple-style dispatch
// ===========================================================================

#[test]
fn maple_jacprod_4arg() {
    let (code, stdout, _) = run(&["-c", "jacprod(1, 5, q, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("q"), "jacprod(1,5,q,20) should produce series in q");
    assert!(stdout.contains("O(q^20)"), "should have truncation");
}

#[test]
fn maple_tripleprod_3arg() {
    let (code, stdout, _) = run(&["-c", "tripleprod(q, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^10)"), "should have truncation at 10");
}

#[test]
fn maple_quinprod_3arg() {
    let (code, stdout, _) = run(&["-c", "quinprod(q, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^10)"), "should have truncation");
}

#[test]
fn maple_winquist_4arg() {
    let (code, _stdout, _) = run(&["-c", "winquist(q, q^2, q, 10)"]);
    assert_eq!(code, 0);
    // winquist should produce a series
}

#[test]
fn maple_qbin_garvan_3arg() {
    let (code, stdout, _) = run(&["-c", "qbin(q, 2, 4)"]);
    assert_eq!(code, 0);
    // qbin(q,2,4) = [4 choose 2]_q = 1 + q + 2*q^2 + q^3 + q^4
    assert!(stdout.contains("q^4 + q^3 + 2*q^2 + q + 1"), "exact polynomial expected (descending)");
    assert!(!stdout.contains("O(q^"), "exact polynomial should not have O() truncation");
}

#[test]
fn maple_qbin_4arg_with_truncation() {
    let (code, stdout, _) = run(&["-c", "qbin(4, 2, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("q"), "should contain series terms");
}

#[test]
fn maple_etaq_multi_delta() {
    let (code, stdout, _) = run(&["-c", "etaq(q, [1, 2], 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^10)"), "should have truncation");
}

#[test]
fn maple_etaq_single_delta() {
    let (code, stdout, _) = run(&["-c", "etaq(q, 3, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^10)"), "should have truncation");
}

#[test]
fn numbpart_primary_name() {
    let (code, stdout, _) = run(&["-c", "numbpart(100)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("190569292"), "numbpart(100) = 190569292");
}

#[test]
fn numbpart_small() {
    let (code, stdout, _) = run(&["-c", "numbpart(5)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("7"), "numbpart(5) = 7");
}

#[test]
fn numbpart_bounded() {
    let (code, stdout, _) = run(&["-c", "numbpart(5, 3)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("5"), "numbpart(5, 3) = 5");
}

#[test]
fn partition_count_alias_still_works() {
    let (code, stdout, _) = run(&["-c", "partition_count(100)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("190569292"), "partition_count alias should still work");
}

#[test]
fn legacy_jacprod_3arg_still_works() {
    let (code, stdout, _) = run(&["-c", "jacprod(1, 5, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^20)"), "legacy jacprod should still work");
}

#[test]
fn legacy_tripleprod_4arg_still_works() {
    let (code, stdout, _) = run(&["-c", "tripleprod(1, 1, 1, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^20)"), "legacy tripleprod should still work");
}

#[test]
fn legacy_qbin_3arg_still_works() {
    let (code, stdout, _) = run(&["-c", "qbin(4, 2, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("q"), "legacy qbin should still work");
}

#[test]
fn legacy_etaq_3arg_still_works() {
    let (code, stdout, _) = run(&["-c", "etaq(1, 1, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("O(q^20)"), "legacy etaq should still work");
}

#[test]
fn numbpart_zero() {
    // numbpart(0) = 1 (empty partition)
    let (code, stdout, _) = run(&["-c", "numbpart(0)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("1"), "numbpart(0) = 1");
}

#[test]
fn numbpart_bounded_zero_max() {
    // numbpart(5, 0) = 0 (no parts allowed)
    let (code, stdout, _) = run(&["-c", "numbpart(5, 0)"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "0", "numbpart(5, 0) = 0");
}

// ===========================================================================
// Phase 35: Series Analysis Signatures -- Maple-style dispatch
// ===========================================================================

#[test]
fn sift_maple_5arg() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(50); sift(f, q, 5, 4, 50)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("q"), "sift should produce series in q");
    // Coefficients of q^(5i+4) from partition_gf: first term p(4)=5
    assert!(stdout.contains("5"), "should contain coefficient 5 (= p(4))");
}

#[test]
fn sift_maple_invalid_residue() {
    let (code, _, stderr) = run(&["-c", "f := partition_gf(50); sift(f, q, 5, 7, 50)"]);
    assert_ne!(code, 0, "sift with k >= n should fail");
    assert!(
        stderr.contains("residue"),
        "error should mention 'residue', got: {}",
        stderr
    );
}

#[test]
fn prodmake_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(30); prodmake(f, q, 15)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("exponents"), "prodmake should return exponents dict");
    assert!(stdout.contains("terms_used"), "prodmake should return terms_used field");
}

#[test]
fn etamake_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(30); etamake(f, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("eta(tau)"), "etamake should display eta notation, got: {}", stdout);
}

#[test]
fn jacprodmake_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := jacprod(1, 5, q, 30); jacprodmake(f, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("factors"), "jacprodmake should return factors");
    assert!(stdout.contains("is_exact"), "jacprodmake should return is_exact flag");
}

#[test]
fn jacprodmake_maple_4arg_with_period() {
    let (code, stdout, _) = run(&["-c", "f := jacprod(1, 5, q, 30); jacprodmake(f, q, 10, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("factors"), "jacprodmake with period filter should return factors");
}

#[test]
fn mprodmake_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := distinct_parts_gf(30); mprodmake(f, q, 10)"]);
    assert_eq!(code, 0);
    // mprodmake returns a plain dict like {1: 1, 2: 1, ...}
    assert!(stdout.contains("1:"), "mprodmake should return a dict with exponents");
}

#[test]
fn qetamake_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(30); qetamake(f, q, 10)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("factors"), "qetamake should return factors");
}

#[test]
fn qfactor_maple_2arg() {
    let (code, stdout, _) = run(&["-c", "f := aqprod(q, q, 5, 20); qfactor(f, q)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("(1-q)"), "qfactor should display product form, got: {}", stdout);
}

#[test]
fn qfactor_maple_3arg() {
    let (code, stdout, _) = run(&["-c", "f := aqprod(q, q, 5, 20); qfactor(f, q, 20)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("(1-q)"), "qfactor 3-arg should display product form, got: {}", stdout);
}

#[test]
fn sift_old_signature_errors() {
    let (code, _, stderr) = run(&["-c", "sift(partition_gf(30), 5, 0)"]);
    assert_ne!(code, 0, "old sift signature should fail");
    assert!(
        stderr.contains("expects 5 arguments"),
        "should report wrong arg count, got: {}",
        stderr
    );
}

#[test]
fn prodmake_old_signature_errors() {
    let (code, _, stderr) = run(&["-c", "prodmake(partition_gf(30), 10)"]);
    assert_ne!(code, 0, "old prodmake signature should fail");
    assert!(
        stderr.contains("expects 3 arguments"),
        "should report wrong arg count, got: {}",
        stderr
    );
}

// ===========================================================================
// Phase 36: Relation Discovery Signatures -- Maple-style dispatch
// ===========================================================================

// ---------------------------------------------------------------------------
// findlincombo with SL labels
// ---------------------------------------------------------------------------

#[test]
fn findlincombo_maple_style() {
    // f = 1*f + 0*g, so output should contain "F1" (the label for f)
    let tmp = write_temp_script(
        "qk_test_findlincombo.qk",
        "f := partition_gf(30):\ng := distinct_parts_gf(30):\nfindlincombo(f, [f, g], [F1, F2], q, 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findlincombo should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("F1"),
        "output should contain label F1. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findlincombo_not_found() {
    let tmp = write_temp_script(
        "qk_test_findlincombo_nf.qk",
        "g := etaq(1, 1, 20):\nh := etaq(2, 1, 20):\nfindlincombo(g, [h], [H], q, 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findlincombo not-found should exit 0. stderr: {}", stderr);
    assert!(
        stdout.contains("NOT A LINEAR COMBO"),
        "should print NOT A LINEAR COMBO. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findlincombo_duplicate_sl_error() {
    let tmp = write_temp_script(
        "qk_test_findlincombo_dup.qk",
        "f := partition_gf(20):\nfindlincombo(f, [f, f], [F, F], q, 0)",
    );
    let (code, _, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_ne!(code, 0, "duplicate SL labels should fail");
    assert!(
        stderr.contains("duplicate label"),
        "should report duplicate label. stderr: {}",
        stderr
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findlincombo_old_signature_error() {
    // Old 3-arg signature should fail
    let (code, _, stderr) = run(&["-c", "f := partition_gf(20); findlincombo(f, [f], 0)"]);
    assert_ne!(code, 0, "old findlincombo signature should fail");
    assert!(
        stderr.contains("expects 5 arguments"),
        "should report wrong arg count. stderr: {}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// findlincombomodp with SL and p before q
// ---------------------------------------------------------------------------

#[test]
fn findlincombomodp_maple_style() {
    let tmp = write_temp_script(
        "qk_test_findlincombomodp.qk",
        "f := partition_gf(30):\nfindlincombomodp(f, [f], [F], 7, q, 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findlincombomodp should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("F"),
        "output should contain label F. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findlincombomodp_non_prime_error() {
    let (code, _, stderr) = run(&["-c", "f := partition_gf(30); findlincombomodp(f, [f], [F], 4, q, 0)"]);
    assert_ne!(code, 0, "non-prime p should fail");
    assert!(
        stderr.contains("not prime"),
        "should report not prime. stderr: {}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// findhom with q parameter
// ---------------------------------------------------------------------------

#[test]
fn findhom_maple_style() {
    // theta3^4 = theta4^4 would need degree 1 among [theta3^4, theta4^4, 1]
    // Use a simpler test: findhom on series that include a zero series => finds X[i]=0
    let tmp = write_temp_script(
        "qk_test_findhom.qk",
        "findhom([theta3(50)^4, theta2(50)^4, theta4(50)^4], q, 1, 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findhom should succeed. stderr: {}", stderr);
    // Should find something (theta2^4 starts at q^1 so X[2] relation)
    assert!(
        stdout.contains("X["),
        "output should contain X[i] labels. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

// ---------------------------------------------------------------------------
// findhommodp with p before q
// ---------------------------------------------------------------------------

#[test]
fn findhommodp_p_before_q() {
    let tmp = write_temp_script(
        "qk_test_findhommodp.qk",
        "e1 := etaq(1, 1, 30):\ne2 := etaq(2, 1, 30):\nfindhommodp([e1, e2], 7, q, 2, 0)",
    );
    let (code, _stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findhommodp should succeed. stderr: {}", stderr);
    std::fs::remove_file(&tmp).ok();
}

// ---------------------------------------------------------------------------
// findmaxind with 2 args
// ---------------------------------------------------------------------------

#[test]
fn findmaxind_two_args() {
    let tmp = write_temp_script(
        "qk_test_findmaxind.qk",
        "e1 := etaq(1, 1, 20):\ne2 := etaq(2, 1, 20):\nfindmaxind([e1, e2], 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findmaxind should succeed. stderr: {}", stderr);
    // Should return [1, 2] (1-based indices of independent series)
    assert!(
        stdout.contains("[1, 2]"),
        "should contain [1, 2] indices. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

// ---------------------------------------------------------------------------
// findpoly with q parameter
// ---------------------------------------------------------------------------

#[test]
fn findpoly_maple_style() {
    let tmp = write_temp_script(
        "qk_test_findpoly.qk",
        "x := theta3(50)^4:\ny := theta2(50)^4:\nfindpoly(x, y, q, 2, 2)",
    );
    let (code, _stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    // Just verify no crash (may or may not find a relation)
    assert_eq!(code, 0, "findpoly should succeed (exit 0). stderr: {}", stderr);
    std::fs::remove_file(&tmp).ok();
}

// ---------------------------------------------------------------------------
// findcong auto-scan (Garvan-style)
// ---------------------------------------------------------------------------

#[test]
fn findcong_garvan_auto_scan() {
    // Use partition_gf(201) to provide T=200 terms (need one extra for boundary)
    let tmp = write_temp_script(
        "qk_test_findcong.qk",
        "p := partition_gf(201):\nfindcong(p, 200)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findcong should succeed. stderr: {}", stderr);
    // Should find Ramanujan's p(5n+4) = 0 mod 5
    assert!(
        stdout.contains("[4, 5, 5]"),
        "should find [4, 5, 5] (Ramanujan congruence). stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findcong_with_lm() {
    // With LM=5, should find mod-5 but NOT mod-7 results
    let tmp = write_temp_script(
        "qk_test_findcong_lm.qk",
        "p := partition_gf(201):\nfindcong(p, 200, 5)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findcong with LM should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("[4, 5, 5]"),
        "should find [4, 5, 5]. stdout: {}",
        stdout
    );
    // LM=5 should exclude modulus 7 results
    assert!(
        !stdout.contains("[5, 7, 7]"),
        "LM=5 should NOT find mod-7 results. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn findcong_old_signature_error() {
    // Old (series, [moduli]) signature should fail -- list is not integer for arg 2
    let (code, _, stderr) = run(&["-c", "p := partition_gf(50); findcong(p, [5, 7])"]);
    assert_ne!(code, 0, "old findcong signature should fail");
    assert!(
        stderr.contains("must be integer") || stderr.contains("got list"),
        "should report type error. stderr: {}",
        stderr
    );
}

// ===========================================================================
// Phase 37: Jacobi Products & Conversions
// ===========================================================================

#[test]
fn jac_creates_jacobi_product() {
    let (code, stdout, stderr) = run(&["-c", "JAC(1,5)"]);
    assert_eq!(code, 0, "JAC(1,5) should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("JAC(1,5)"),
        "expected JAC(1,5) in output: {}",
        stdout
    );
}

#[test]
fn jac_multiply_combines_factors() {
    let (code, stdout, stderr) = run(&["-c", "JAC(1,5) * JAC(2,5)"]);
    assert_eq!(code, 0, "JAC multiply should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("JAC(1,5)*JAC(2,5)"),
        "expected combined product: {}",
        stdout
    );
}

#[test]
fn theta_numeric_z() {
    // theta(1, q, 5) = sum(q^(i^2), i=-5..5) for i^2 < 5: i in {-2,-1,0,1,2}
    // q^0: 1 (i=0), q^1: 2 (i=+-1), q^4: 2 (i=+-2)
    let (code, stdout, stderr) = run(&["-c", "theta(1, q, 5)"]);
    assert_eq!(code, 0, "theta should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("2*q^4 + 2*q + 1"),
        "expected theta coefficients (descending): {}",
        stdout
    );
}

#[test]
fn jac2series_single_factor() {
    // jac2series(JAC(1,1), q, 10) should produce (q;q)_inf = 1 - q - q^2 + q^5 + q^7 + O(q^10)
    let (code, stdout, stderr) = run(&["-c", "jac2series(JAC(1,1), q, 10)"]);
    assert_eq!(code, 0, "jac2series should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("q^7 + q^5"),
        "expected (q;q)_inf descending series start: {}",
        stdout
    );
}

#[test]
fn jac2series_wrong_type_errors() {
    let (code, _, stderr) = run(&["-c", "jac2series(42, q, 10)"]);
    assert_ne!(code, 0, "jac2series with integer should fail");
    assert!(
        stderr.contains("expected Jacobi product"),
        "expected type error: {}",
        stderr
    );
}

#[test]
fn jac2prod_shows_product_notation() {
    let (code, stdout, stderr) = run(&["-c", "jac2prod(JAC(1,5), q, 20)"]);
    assert_eq!(code, 0, "jac2prod should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("(1-q)"),
        "expected product notation: {}",
        stdout
    );
}

#[test]
fn qs2jaccombo_finds_decomposition() {
    // etaq(q, 1, 30) = (q;q)_inf; qs2jaccombo should find a JAC decomposition
    let tmp = write_temp_script(
        "qk_test_qs2jaccombo.qk",
        "f := etaq(q, 1, 30):\nqs2jaccombo(f, q, 30)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "qs2jaccombo should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("JAC"),
        "expected JAC decomposition: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn jac_invalid_b_zero_errors() {
    // JAC(1,0) should fail since b must be positive
    let (code, _, stderr) = run(&["-c", "JAC(1, 0)"]);
    assert_ne!(code, 0, "JAC(1,0) should fail");
    assert!(
        stderr.contains("positive") || stderr.contains("greater than"),
        "expected validation error: {}",
        stderr
    );
}

#[test]
fn theta_monomial_z() {
    // theta(q^2, q, 10) = sum(q^(2i+i^2), i=-T..T)
    let (code, stdout, stderr) = run(&["-c", "theta(q^2, q, 10)"]);
    assert_eq!(code, 0, "theta with monomial z should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("q"),
        "expected series output: {}",
        stdout
    );
}

#[test]
fn jac2series_matches_etaq() {
    // jac2series(JAC(1,5), q, 15) should produce same result as etaq(1, 5, 15) (legacy form)
    let tmp = write_temp_script(
        "qk_test_jac2series_match.qk",
        "a := jac2series(JAC(1,5), q, 15):\nb := etaq(1, 5, 15):\na - b",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "jac2series cross-check should succeed. stderr: {}", stderr);
    // Difference should be 0 (or O(q^15))
    assert!(
        stdout.contains("O(q^15)") || stdout.contains("0"),
        "expected zero difference: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

// ===========================================================================
// Phase 38: Analysis & Discovery Functions
// ===========================================================================

#[test]
fn cli_lqdegree0_partition_gf() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(20); lqdegree0(f)"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("0"),
        "lqdegree0 of partition_gf should be 0, got: {}",
        stdout
    );
}

#[test]
fn cli_checkmult_not_multiplicative() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(50); checkmult(f, 30)"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("NOT MULTIPLICATIVE"),
        "partition function should not be multiplicative, got: {}",
        stdout
    );
    assert!(
        stdout.contains("0"),
        "should return 0, got: {}",
        stdout
    );
}

#[test]
fn cli_checkmult_with_yes() {
    let (code, stdout, _) = run(&["-c", "f := partition_gf(50); checkmult(f, 30, 'yes')"]);
    assert_eq!(code, 0);
    // Should print multiple NOT MULTIPLICATIVE lines
    let not_mult_count = stdout.matches("NOT MULTIPLICATIVE").count();
    assert!(
        not_mult_count >= 2,
        "with 'yes' should print multiple failures, got {} lines",
        not_mult_count
    );
}

#[test]
fn cli_checkprod_eta_nice() {
    let (code, stdout, _) = run(&["-c", "f := etaq(1, 1, 30); checkprod(f, 10, 30)"]);
    assert_eq!(code, 0);
    // eta is a nice product, should contain [_, 1]
    assert!(
        stdout.contains("1]"),
        "eta should be a nice product, got: {}",
        stdout
    );
}

#[test]
fn cli_findprod_garvan_4arg() {
    let (code, stdout, _) = run(&["-c", "e1 := etaq(1, 1, 30); e2 := etaq(2, 1, 30); findprod([e1, e2], 1, 10, 30)"]);
    assert_eq!(code, 0);
    // Should return a list (possibly with results)
    assert!(
        stdout.contains("["),
        "findprod should return a list, got: {}",
        stdout
    );
}

#[test]
fn cli_findprod_old_3arg_errors() {
    let (code, _, stderr) = run(&["-c", "e1 := etaq(1, 1, 20); findprod([e1], 2, 5)"]);
    assert_ne!(code, 0, "old 3-arg findprod should error");
    assert!(
        stderr.contains("expects 4 arguments") || stderr.contains("expected 4"),
        "old 3-arg findprod should error, got stderr: {}",
        stderr
    );
}

// ===========================================================================
// Backward Compatibility: v1.x signatures (COMPAT-01, COMPAT-02)
// ===========================================================================

// ---------------------------------------------------------------------------
// Product/theta functions -- legacy (v1.x) signatures
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_etaq_legacy_3arg() {
    // Legacy: etaq(b, t, order) with numeric args
    let (code, stdout, stderr) = run(&["-c", "etaq(1, 1, 20)"]);
    assert_eq!(code, 0, "legacy etaq(1,1,20) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
    assert!(stdout.contains("q"), "should contain q terms, got: {}", stdout);
    // Verify actual coefficients: (q;q)_inf starts with 1 - q - q^2 + q^5 + q^7 - ...
    assert!(stdout.contains("1"), "should contain constant term 1");
}

#[test]
fn backward_compat_aqprod_legacy_5arg() {
    // Legacy: aqprod(cn, cd, power, infinity, order)
    let (code, stdout, stderr) = run(&["-c", "aqprod(1, 1, 1, infinity, 20)"]);
    assert_eq!(code, 0, "legacy aqprod 5-arg should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
    // (q;q)_inf = 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + ...
    assert!(stdout.contains("q"), "should contain q terms");
}

#[test]
fn backward_compat_aqprod_legacy_5arg_finite() {
    // Legacy: aqprod(cn, cd, power, n, order) with finite n
    let (code, stdout, stderr) = run(&["-c", "aqprod(1, 1, 2, 5, 20)"]);
    assert_eq!(code, 0, "legacy aqprod finite should succeed. stderr: {}", stderr);
    // Should produce a series or polynomial output
    assert!(!stdout.trim().is_empty(), "should produce output");
    assert!(stdout.contains("q"), "should contain q terms, got: {}", stdout);
}

#[test]
fn backward_compat_jacprod_legacy_3arg() {
    // Legacy: jacprod(a, b, order)
    let (code, stdout, stderr) = run(&["-c", "jacprod(1, 5, 20)"]);
    assert_eq!(code, 0, "legacy jacprod(1,5,20) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
    // J(1,5) = (q;q^5)_inf series
    assert!(stdout.contains("q"), "should contain q terms");
}

#[test]
fn backward_compat_tripleprod_legacy_4arg() {
    // Legacy: tripleprod(cn, cd, power, order)
    let (code, stdout, stderr) = run(&["-c", "tripleprod(1, 1, 1, 20)"]);
    assert_eq!(code, 0, "legacy tripleprod should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
}

#[test]
fn backward_compat_quinprod_legacy_4arg() {
    // Legacy: quinprod(cn, cd, power, order)
    let (code, stdout, stderr) = run(&["-c", "quinprod(1, 1, 1, 20)"]);
    assert_eq!(code, 0, "legacy quinprod should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
}

#[test]
fn backward_compat_winquist_legacy_7arg() {
    // Legacy: winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)
    let (code, stdout, stderr) = run(&["-c", "winquist(1, 1, 1, 1, 1, 1, 20)"]);
    assert_eq!(code, 0, "legacy winquist 7-arg should succeed. stderr: {}", stderr);
    // Winquist product may be all-zero within truncation range
    assert!(!stdout.trim().is_empty(), "should produce output");
}

#[test]
fn backward_compat_qbin_legacy_3arg() {
    // Legacy: qbin(n, k, order) -- [n choose k]_q polynomial
    let (code, stdout, stderr) = run(&["-c", "qbin(4, 2, 20)"]);
    assert_eq!(code, 0, "legacy qbin(4,2,20) should succeed. stderr: {}", stderr);
    // [4 choose 2]_q = 1 + q + 2*q^2 + q^3 + q^4
    assert!(stdout.contains("q"), "should contain q terms, got: {}", stdout);
    assert!(stdout.contains("q^4"), "should contain q^4 term");
    assert!(stdout.contains("2*q^2"), "should contain 2*q^2 coefficient");
}

#[test]
fn backward_compat_numbpart_alias() {
    // partition_count is old name, numbpart is new canonical name
    let (code, stdout, stderr) = run(&["-c", "partition_count(100)"]);
    assert_eq!(code, 0, "partition_count alias should succeed. stderr: {}", stderr);
    assert!(stdout.contains("190569292"), "partition_count(100) should be 190569292, got: {}", stdout);
}

#[test]
fn backward_compat_numbpart_primary() {
    // numbpart is now the primary name
    let (code, stdout, _) = run(&["-c", "numbpart(5)"]);
    assert_eq!(code, 0, "numbpart(5) should succeed");
    assert_eq!(stdout.trim(), "7", "numbpart(5) = 7, got: {}", stdout);
}

// ---------------------------------------------------------------------------
// Series analysis functions -- Maple signatures (Phase 35 changes)
// Note: Phase 35 broke backward compat for sift/prodmake/etamake etc.
// Old signatures intentionally error. Tests here verify NEW Maple signatures.
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_sift_maple_5arg() {
    let (code, stdout, stderr) = run(&["-c", "f := partition_gf(50); sift(f, q, 5, 0, 50)"]);
    assert_eq!(code, 0, "sift maple 5-arg should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q"), "sift should produce series in q, got: {}", stdout);
    // p(0)=1 is the first coefficient at q^0 in the sifted series
    assert!(stdout.contains("1"), "should contain coefficient 1");
}

#[test]
fn backward_compat_prodmake_maple_3arg() {
    let (code, stdout, stderr) = run(&["-c", "f := partition_gf(30); prodmake(f, q, 15)"]);
    assert_eq!(code, 0, "prodmake maple 3-arg should succeed. stderr: {}", stderr);
    assert!(stdout.contains("exponents"), "prodmake should return exponents dict, got: {}", stdout);
}

#[test]
fn backward_compat_etamake_maple_3arg() {
    let (code, stdout, stderr) = run(&["-c", "f := partition_gf(30); etamake(f, q, 10)"]);
    assert_eq!(code, 0, "etamake maple 3-arg should succeed. stderr: {}", stderr);
    assert!(stdout.contains("eta(tau)"), "etamake should display eta notation, got: {}", stdout);
}

#[test]
fn backward_compat_qfactor_maple_2arg() {
    let (code, stdout, stderr) = run(&["-c", "f := aqprod(q, q, 5, 20); qfactor(f, q)"]);
    assert_eq!(code, 0, "qfactor maple 2-arg should succeed. stderr: {}", stderr);
    assert!(stdout.contains("(1-q)"), "qfactor should display product form, got: {}", stdout);
}

// ---------------------------------------------------------------------------
// Relation discovery functions -- Garvan signatures (Phase 36 changes)
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_findlincombo_garvan() {
    let tmp = write_temp_script(
        "qk_test_bc_findlincombo.qk",
        "f := partition_gf(30):\ng := distinct_parts_gf(30):\nfindlincombo(f, [f, g], [F1, F2], q, 0)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findlincombo garvan should succeed. stderr: {}", stderr);
    assert!(
        stdout.contains("F1"),
        "output should contain label F1. stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn backward_compat_findcong_garvan() {
    let tmp = write_temp_script(
        "qk_test_bc_findcong.qk",
        "p := partition_gf(201):\nfindcong(p, 200)",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "findcong garvan should succeed. stderr: {}", stderr);
    // Should find Ramanujan's p(5n+4) = 0 mod 5
    assert!(
        stdout.contains("[4, 5, 5]"),
        "should find [4, 5, 5] (Ramanujan congruence). stdout: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

// ---------------------------------------------------------------------------
// New functions (Phases 37-38) -- no legacy to preserve, verify they work
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_theta_new() {
    let (code, stdout, stderr) = run(&["-c", "theta(1, q, 5)"]);
    assert_eq!(code, 0, "theta should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q"), "theta should produce series in q, got: {}", stdout);
    // theta(1, q, 5) = 2*q^4 + 2*q + 1 + O(q^5)
    assert!(stdout.contains("2*q^4"), "should contain 2*q^4 term, got: {}", stdout);
}

#[test]
fn backward_compat_jac2series_new() {
    let (code, stdout, stderr) = run(&["-c", "jac2series(JAC(1,1), q, 10)"]);
    assert_eq!(code, 0, "jac2series should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q"), "jac2series should produce series in q, got: {}", stdout);
    // (q;q)_inf = 1 - q - q^2 + q^5 + q^7 + O(q^10)
    assert!(stdout.contains("O(q^10)"), "should have O(q^10), got: {}", stdout);
}

#[test]
fn backward_compat_checkmult_new() {
    let (code, _stdout, stderr) = run(&["-c", "f := partition_gf(50); checkmult(f, 30)"]);
    assert_eq!(code, 0, "checkmult should succeed. stderr: {}", stderr);
    // checkmult returns 0 for non-multiplicative
}

#[test]
fn backward_compat_lqdegree0_new() {
    let (code, stdout, stderr) = run(&["-c", "f := partition_gf(20); lqdegree0(f)"]);
    assert_eq!(code, 0, "lqdegree0 should succeed. stderr: {}", stderr);
    assert!(stdout.contains("0"), "lqdegree0 of partition_gf should be 0, got: {}", stdout);
}

// ---------------------------------------------------------------------------
// Cross-validation: legacy vs Garvan signatures produce identical results
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_etaq_legacy_matches_garvan() {
    // Compute both legacy etaq(1,1,20) and Garvan etaq(q,1,20) and subtract
    let tmp = write_temp_script(
        "qk_test_bc_etaq_crossval.qk",
        "a := etaq(1, 1, 20):\nb := etaq(q, 1, 20):\na - b",
    );
    let (code, stdout, stderr) = run(&[tmp.to_str().unwrap()]);
    assert_eq!(code, 0, "cross-validation should succeed. stderr: {}", stderr);
    // Difference should be zero -- only truncation marker remains
    assert!(
        stdout.trim() == "O(q^20)" || stdout.trim() == "0",
        "legacy and Garvan should produce identical results, difference should be O(q^20) or 0, got: {}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

// --- Phase 50: jac2series 2-arg and quinprod identity modes ---

#[test]
fn jac2series_2arg_garvan() {
    // jac2series(JAC(1,5), 20) -- Garvan 2-arg form with triple product
    let (code, stdout, stderr) = run(&["-c", "jac2series(JAC(1,5), 20)"]);
    assert_eq!(code, 0, "jac2series 2-arg should succeed. stderr: {}", stderr);
    // Triple product JAC(1,5) = (q;q^5)(q^4;q^5)(q^5;q^5), starts with 1 - q - q^4 + ...
    assert!(stdout.contains("1"), "should contain constant term 1, got: {}", stdout);
    assert!(stdout.contains("q"), "should contain q terms, got: {}", stdout);
    assert!(stdout.contains("O(q^20)"), "should have O(q^20), got: {}", stdout);
}

#[test]
fn jac2series_2arg_jac0() {
    // jac2series(JAC(0,1), 10) -- Euler function (q;q)_inf
    let (code, stdout, stderr) = run(&["-c", "jac2series(JAC(0,1), 10)"]);
    assert_eq!(code, 0, "jac2series JAC(0,1) should succeed. stderr: {}", stderr);
    // (q;q)_inf = 1 - q - q^2 + q^5 + q^7 + O(q^10)
    assert!(stdout.contains("q^7"), "Euler function should have q^7, got: {}", stdout);
    assert!(stdout.contains("q^5"), "Euler function should have q^5, got: {}", stdout);
    assert!(stdout.contains("O(q^10)"), "should have O(q^10), got: {}", stdout);
}

#[test]
fn quinprod_prodid() {
    let (code, stdout, stderr) = run(&["-c", "quinprod(z, q, prodid)"]);
    assert_eq!(code, 0, "quinprod prodid should succeed. stderr: {}", stderr);
    assert!(stdout.contains("(z"), "prodid output should reference z, got: {}", stdout);
    assert!(stdout.contains("(q,q)_inf"), "prodid should contain (q,q)_inf, got: {}", stdout);
}

#[test]
fn quinprod_seriesid() {
    let (code, stdout, stderr) = run(&["-c", "quinprod(z, q, seriesid)"]);
    assert_eq!(code, 0, "quinprod seriesid should succeed. stderr: {}", stderr);
    assert!(stdout.contains("sum"), "seriesid should contain 'sum', got: {}", stdout);
    assert!(stdout.contains("3*m"), "seriesid should contain '3*m', got: {}", stdout);
}

// =========================================================================
// Phase 50 Plan 02: Subscript variables, multi-arg subs, theta monomial, radsimp
// =========================================================================

#[test]
fn subscript_variable_assignment() {
    let (code, stdout, stderr) = run(&["-c", "X[1] := 5; X[1]"]);
    assert_eq!(code, 0, "subscript variable assignment should succeed. stderr: {}", stderr);
    assert!(stdout.contains("5"), "X[1] should evaluate to 5, got: {}", stdout);
}

#[test]
fn subs_multi_indexed() {
    // Single substitution backward compat
    let (code, stdout, stderr) = run(&["-c", "subs(q=0, 1 + q + q^2)"]);
    assert_eq!(code, 0, "subs single sub should succeed. stderr: {}", stderr);
    assert!(stdout.contains("1"), "subs(q=0, 1+q+q^2) should be 1, got: {}", stdout);
}

#[test]
fn theta3_monomial() {
    // theta3(q^2, 10) should produce a series with only even-exponent terms
    let (code, stdout, stderr) = run(&["-c", "theta3(q^2, 10)"]);
    assert_eq!(code, 0, "theta3(q^2, 10) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q^2"), "theta3(q^2,10) should have q^2 term, got: {}", stdout);
    assert!(stdout.contains("q^8"), "theta3(q^2,10) should have q^8 term, got: {}", stdout);
    // Check no odd exponents
    assert!(!stdout.contains("q^1 ") && !stdout.contains("q^3 ") && !stdout.contains("q^5 "),
        "theta3(q^2,10) should have no odd-exponent terms, got: {}", stdout);
}

#[test]
fn radsimp_series() {
    // radsimp(theta3(q,20)) should return the same as theta3(q,20)
    let (code1, stdout1, _) = run(&["-c", "theta3(q, 20)"]);
    let (code2, stdout2, stderr2) = run(&["-c", "radsimp(theta3(q, 20))"]);
    assert_eq!(code1, 0);
    assert_eq!(code2, 0, "radsimp should succeed. stderr: {}", stderr2);
    assert_eq!(stdout1.trim(), stdout2.trim(), "radsimp(theta3(q,20)) should equal theta3(q,20)");
}

#[test]
fn radsimp_quotient() {
    // radsimp(theta3(q,50)/theta3(q^5,10)) should produce a non-trivial series
    let (code, stdout, stderr) = run(&["-c", "radsimp(theta3(q, 50) / theta3(q^5, 10))"]);
    assert_eq!(code, 0, "radsimp quotient should succeed. stderr: {}", stderr);
    assert!(stdout.contains("q"), "quotient should contain q terms, got: {}", stdout);
    assert!(stdout.contains("O(q^"), "quotient should have truncation, got: {}", stdout);
}
