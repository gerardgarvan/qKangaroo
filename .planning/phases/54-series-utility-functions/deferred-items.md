# Deferred Items - Phase 54

## Pre-existing Test Failure

- **Test:** `err_05_read_nonexistent_shows_file_not_found` in cli_integration.rs:783
- **Issue:** Parser rejects `.` in filenames (`read("/nonexistent/file.qk")` fails with "unexpected character '.'")
- **Scope:** Pre-existing issue, not related to Phase 54 changes
- **Impact:** Low -- only affects `.` in string-argument filenames passed to `read()`
