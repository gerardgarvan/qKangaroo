//! REPL helper for the q-Kangaroo interactive shell.
//!
//! Provides [`ReplHelper`] which implements rustyline's `Helper` composite
//! trait: tab completion (functions with auto-paren, session commands at line
//! start, user-defined variables), bracket-counting multi-line validation,
//! and no-op highlighter/hinter.

use rustyline::completion::{Completer, Pair};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper, Highlighter, Hinter};

// ---------------------------------------------------------------------------
// ReplHelper
// ---------------------------------------------------------------------------

/// Line-editing helper with tab completion and bracket validation.
///
/// - **Functions:** All canonical function names auto-complete with `(`.
/// - **Keywords:** Scripting keywords (`for`, `proc`, `if`, etc.) complete
///   without trailing `(`.
/// - **Commands:** `help`, `quit`, `exit`, `clear`, `set` complete at line start.
/// - **Variables:** User-defined names synced after each eval via
///   [`update_var_names`](ReplHelper::update_var_names).
/// - **Validator:** Counts `(` / `[` depth; returns `Incomplete` when positive.
#[derive(Helper, Highlighter, Hinter)]
pub struct ReplHelper {
    // NOTE: Completer and Validator are manually implemented below.
    // Highlighter and Hinter use derive (no-op defaults).
    /// Canonical function names (static, from eval.rs ALL_FUNCTION_NAMES).
    function_names: Vec<&'static str>,
    /// Language keyword names (complete without trailing paren).
    keyword_names: Vec<&'static str>,
    /// Session command names for completion.
    command_names: Vec<&'static str>,
    /// User-defined variable names (updated after each eval).
    var_names: Vec<String>,
}

impl ReplHelper {
    /// Create a new helper with all canonical function names and commands.
    pub fn new() -> Self {
        Self {
            function_names: Self::canonical_function_names(),
            keyword_names: vec![
                "for", "from", "to", "by", "do", "od",
                "if", "then", "elif", "else", "fi",
                "proc", "local", "end",
                "RETURN",
                "and", "or", "not",
            ],
            command_names: vec!["help", "quit", "exit", "clear", "restart", "set", "latex", "save"],
            var_names: Vec::new(),
        }
    }

    /// Update the set of user-defined variable names for tab completion.
    ///
    /// Called after each successful eval in the main REPL loop.
    pub fn update_var_names(&mut self, var_names: Vec<String>) {
        self.var_names = var_names;
    }

    /// All 101 canonical function names -- must match eval.rs ALL_FUNCTION_NAMES
    /// exactly. NO Maple aliases.
    fn canonical_function_names() -> Vec<&'static str> {
        vec![
            // Group 1: Products (7)
            "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
            // Group 2: Partitions (7)
            "numbpart", "partition_gf", "distinct_parts_gf", "odd_parts_gf",
            "bounded_parts_gf", "rank_gf", "crank_gf",
            // Group 3: Theta (3)
            "theta2", "theta3", "theta4",
            // Group 4: Analysis (12)
            "sift", "qdegree", "lqdegree", "lqdegree0", "qfactor",
            "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
            "checkmult", "checkprod",
            // Group 5: Relations (12)
            "findlincombo", "findhomcombo", "findnonhomcombo",
            "findlincombomodp", "findhomcombomodp",
            "findhom", "findnonhom", "findhommodp",
            "findmaxind", "findprod", "findcong", "findpoly",
            // Group 6: Hypergeometric (9)
            "phi", "psi", "try_summation",
            "heine1", "heine2", "heine3",
            "sears_transform", "watson_transform", "find_transformation_chain",
            // Group 7: Mock Theta / Appell-Lerch / Bailey (27)
            "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3",
            "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
            "mock_theta_f0_5", "mock_theta_f1_5",
            "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
            "mock_theta_phi0_5", "mock_theta_phi1_5",
            "mock_theta_psi0_5", "mock_theta_psi1_5",
            "mock_theta_chi0_5", "mock_theta_chi1_5",
            "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
            "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
            "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain", "bailey_discover",
            // Group 8: Identity Proving (7)
            "prove_eta_id", "search_identities",
            "q_gosper", "q_zeilberger", "verify_wz", "q_petkovsek",
            "prove_nonterminating",
            // Group 9: Variable Management (2)
            "anames", "restart",
            // Group 10: Jacobi Products (5)
            "JAC", "theta", "jac2prod", "jac2series", "qs2jaccombo",
            // Group Q: Expression Operations (2)
            "series", "expand",
            // Group R: Polynomial Operations (2)
            "factor", "subs",
            // Group P: Number Theory (4)
            "floor", "legendre", "min", "max",
            // Group T: Simplification (1)
            "radsimp",
            // Group M: Script Loading (1)
            "read",
        ]
    }

    /// Core completion logic (separated from rustyline types for testability).
    ///
    /// Returns `(word_start, candidates)` where each candidate is
    /// `(display, replacement)`.
    fn complete_inner(&self, line: &str, pos: usize) -> (usize, Vec<(String, String)>) {
        // Find the word start: scan backwards for non-alphanumeric/underscore.
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];

        if prefix.is_empty() {
            return (start, vec![]);
        }

        // Check if next char is already '(' (avoid double-paren).
        let has_paren_after = line.get(pos..pos + 1) == Some("(");

        let mut candidates = Vec::new();

        // Complete function names (with auto-paren).
        for &name in &self.function_names {
            if name.starts_with(prefix) {
                let replacement = if has_paren_after {
                    name.to_string()
                } else {
                    format!("{}(", name)
                };
                candidates.push((name.to_string(), replacement));
            }
        }

        // Complete keyword names (without auto-paren).
        for &kw in &self.keyword_names {
            if kw.starts_with(prefix) {
                candidates.push((kw.to_string(), kw.to_string()));
            }
        }

        // Complete session commands (only at start of line, no paren).
        if start == 0 {
            for &cmd in &self.command_names {
                if cmd.starts_with(prefix) {
                    candidates.push((cmd.to_string(), cmd.to_string()));
                }
            }
        }

        // Complete user-defined variable names (no paren).
        for var_name in &self.var_names {
            if var_name.starts_with(prefix) {
                candidates.push((var_name.clone(), var_name.clone()));
            }
        }

        (start, candidates)
    }

    /// Core bracket/keyword-counting logic (separated for testability).
    ///
    /// Returns `true` if the input has unclosed brackets or unclosed
    /// control flow blocks (for/od, if/fi).
    fn is_incomplete(input: &str) -> bool {
        let mut bracket_depth: i32 = 0;
        let mut for_depth: i32 = 0;
        let mut if_depth: i32 = 0;
        let mut proc_depth: i32 = 0;
        let mut word = String::new();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_comment = false;

        for ch in input.chars() {
            // Handle comment state: skip until newline
            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                continue;
            }

            // Handle string literal state
            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }

            // Start of string literal
            if ch == '"' || ch == '\'' {
                Self::check_keyword(&word, &mut for_depth, &mut if_depth, &mut proc_depth);
                word.clear();
                in_string = true;
                string_char = ch;
                continue;
            }

            // Start of comment
            if ch == '#' {
                Self::check_keyword(&word, &mut for_depth, &mut if_depth, &mut proc_depth);
                word.clear();
                in_comment = true;
                continue;
            }

            if ch.is_ascii_alphanumeric() || ch == '_' {
                word.push(ch);
            } else {
                Self::check_keyword(&word, &mut for_depth, &mut if_depth, &mut proc_depth);
                word.clear();
                match ch {
                    '(' | '[' => bracket_depth += 1,
                    ')' | ']' => bracket_depth -= 1,
                    _ => {}
                }
            }
        }
        // Flush final word
        Self::check_keyword(&word, &mut for_depth, &mut if_depth, &mut proc_depth);

        bracket_depth > 0 || for_depth > 0 || if_depth > 0 || proc_depth > 0
    }

    fn check_keyword(word: &str, for_depth: &mut i32, if_depth: &mut i32, proc_depth: &mut i32) {
        match word {
            "for" => *for_depth += 1,
            "od" => *for_depth -= 1,
            "if" => *if_depth += 1,
            "fi" => *if_depth -= 1,
            "proc" => *proc_depth += 1,
            "end" => *proc_depth -= 1,
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Completer
// ---------------------------------------------------------------------------

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, candidates) = self.complete_inner(line, pos);
        let pairs = candidates
            .into_iter()
            .map(|(display, replacement)| Pair {
                display,
                replacement,
            })
            .collect();
        Ok((start, pairs))
    }
}

// ---------------------------------------------------------------------------
// Validator (bracket-counting multi-line)
// ---------------------------------------------------------------------------

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        if Self::is_incomplete(ctx.input()) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical function list must have exactly 101 entries,
    /// matching eval.rs ALL_FUNCTION_NAMES.
    #[test]
    fn canonical_function_count() {
        let names = ReplHelper::canonical_function_names();
        assert_eq!(
            names.len(),
            101,
            "expected 101 canonical function names, got {}",
            names.len()
        );
    }

    /// No duplicates in the canonical list.
    #[test]
    fn no_duplicate_function_names() {
        let names = ReplHelper::canonical_function_names();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "duplicate function names found");
    }

    // -- Completion tests (via complete_inner) -----------------------------

    #[test]
    fn complete_aq_returns_aqprod_with_paren() {
        let h = ReplHelper::new();
        let (start, pairs) = h.complete_inner("aq", 2);
        assert_eq!(start, 0);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "aqprod");
        assert_eq!(pairs[0].1, "aqprod(");
    }

    #[test]
    fn complete_theta_returns_four_candidates() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("theta", 5);
        assert_eq!(pairs.len(), 4);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"theta"));
        assert!(displays.contains(&"theta2"));
        assert!(displays.contains(&"theta3"));
        assert!(displays.contains(&"theta4"));
    }

    #[test]
    fn complete_q_at_start_includes_commands() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("q", 1);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        // Should include function names starting with 'q' AND command "quit"
        assert!(displays.contains(&"quit"), "quit command missing");
        assert!(displays.contains(&"qbin"), "qbin function missing");
    }

    #[test]
    fn complete_q_mid_line_excludes_commands() {
        let h = ReplHelper::new();
        // "f(q" -- cursor is at position 3, word starts at position 2 (after '(')
        let (start, pairs) = h.complete_inner("f(q", 3);
        assert_eq!(start, 2);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        // Should NOT include commands since word doesn't start at position 0
        assert!(!displays.contains(&"quit"), "quit should not appear mid-line");
        // Should include function names starting with 'q'
        assert!(displays.contains(&"qbin"), "qbin function missing");
    }

    #[test]
    fn complete_variable_after_update() {
        let mut h = ReplHelper::new();
        h.update_var_names(vec!["foo".to_string(), "fbar".to_string()]);
        let (_, pairs) = h.complete_inner("fo", 2);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        // "fo" matches keywords "for" and "from", plus variable "foo"
        assert!(displays.contains(&"foo"), "variable 'foo' should appear");
        let foo_pair = pairs.iter().find(|p| p.0 == "foo").unwrap();
        assert_eq!(foo_pair.1, "foo"); // no paren for variables
    }

    #[test]
    fn complete_has_paren_after_no_double() {
        let h = ReplHelper::new();
        // User typed "aqprod(" but cursor is right before the '('
        let (_, pairs) = h.complete_inner("aqprod(", 6);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "aqprod");
        assert_eq!(pairs[0].1, "aqprod"); // no extra '('
    }

    #[test]
    fn complete_empty_prefix_returns_nothing() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("", 0);
        assert!(pairs.is_empty());
    }

    #[test]
    fn complete_no_maple_aliases() {
        let h = ReplHelper::new();
        // partition_count is now an alias and should NOT appear in completions
        let (_, pairs) = h.complete_inner("part", 4);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(!displays.contains(&"partition_count"), "alias partition_count should not appear");
    }

    #[test]
    fn complete_numbpart_canonical() {
        let h = ReplHelper::new();
        // numbpart is now canonical and SHOULD appear in completions
        let (_, pairs) = h.complete_inner("numb", 4);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"numbpart"), "canonical numbpart should appear");
    }

    // -- Validator tests (via is_incomplete) --------------------------------

    #[test]
    fn validator_balanced_parens_valid() {
        assert!(!ReplHelper::is_incomplete("f(1, 2)"));
    }

    #[test]
    fn validator_unclosed_paren_incomplete() {
        assert!(ReplHelper::is_incomplete("f(1, 2"));
    }

    #[test]
    fn validator_nested_balanced_valid() {
        assert!(!ReplHelper::is_incomplete("f(1) + g(2)"));
    }

    #[test]
    fn validator_empty_input_valid() {
        assert!(!ReplHelper::is_incomplete(""));
    }

    #[test]
    fn validator_bracket_incomplete() {
        assert!(ReplHelper::is_incomplete("f([1, 2"));
    }

    #[test]
    fn complete_latex_command() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("lat", 3);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"latex"), "should complete 'lat' to 'latex'");
    }

    #[test]
    fn complete_save_command() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("sav", 3);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"save"), "should complete 'sav' to 'save'");
    }

    // -- Keyword nesting tests ------------------------------------------------

    #[test]
    fn validator_for_incomplete() {
        assert!(ReplHelper::is_incomplete("for n from 1 to 5 do"));
    }

    #[test]
    fn validator_for_complete() {
        assert!(!ReplHelper::is_incomplete("for n from 1 to 5 do n od"));
    }

    #[test]
    fn validator_if_incomplete() {
        assert!(ReplHelper::is_incomplete("if x > 0 then"));
    }

    #[test]
    fn validator_if_complete() {
        assert!(!ReplHelper::is_incomplete("if x > 0 then 1 fi"));
    }

    #[test]
    fn validator_if_elif_incomplete() {
        assert!(ReplHelper::is_incomplete("if x > 0 then 1 elif x = 0 then"));
    }

    #[test]
    fn validator_nested_for_if_incomplete() {
        // fi closes the if, but od is still missing
        assert!(ReplHelper::is_incomplete("for n from 1 to 3 do if n > 1 then n fi"));
    }

    #[test]
    fn validator_nested_for_if_complete() {
        assert!(!ReplHelper::is_incomplete("for n from 1 to 3 do if n > 1 then n fi od"));
    }

    #[test]
    fn validator_keyword_in_comment_ignored() {
        assert!(!ReplHelper::is_incomplete("# for"));
    }

    #[test]
    fn validator_keyword_in_string_ignored() {
        assert!(!ReplHelper::is_incomplete("\"for\""));
    }

    #[test]
    fn validator_multiline_for() {
        assert!(ReplHelper::is_incomplete("for n from 1 to 5 do\n  n;\n"));
        assert!(!ReplHelper::is_incomplete("for n from 1 to 5 do\n  n;\nod"));
    }

    // -- Procedure multiline tests -------------------------------------------

    #[test]
    fn validator_proc_incomplete() {
        assert!(ReplHelper::is_incomplete("f := proc(n)"));
    }

    #[test]
    fn validator_proc_complete() {
        assert!(!ReplHelper::is_incomplete("f := proc(n) n end"));
    }

    #[test]
    fn validator_proc_multiline() {
        assert!(ReplHelper::is_incomplete("f := proc(n)\n  n;\n"));
        assert!(!ReplHelper::is_incomplete("f := proc(n)\n  n;\nend"));
    }

    // -- Keyword completion tests ---------------------------------------------

    #[test]
    fn complete_for_keyword() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("fo", 2);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"for"), "should complete 'fo' to 'for'");
        // Verify keyword completes without trailing paren
        let for_pair = pairs.iter().find(|p| p.0 == "for").unwrap();
        assert_eq!(for_pair.1, "for", "keyword 'for' should not have trailing paren");
    }

    #[test]
    fn complete_proc_keyword() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("pr", 2);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"proc"), "'proc' should be among 'pr' completions");
        // Also check that function names like prove_eta_id are present
        assert!(displays.contains(&"prove_eta_id"), "prove_eta_id should also match 'pr'");
    }

    #[test]
    fn complete_if_keyword() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("if", 2);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"if"), "should complete 'if' to 'if'");
        let if_pair = pairs.iter().find(|p| p.0 == "if").unwrap();
        assert_eq!(if_pair.1, "if", "keyword 'if' should not have trailing paren");
    }

    #[test]
    fn complete_return_keyword() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("RET", 3);
        let displays: Vec<&str> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(displays.contains(&"RETURN"), "should complete 'RET' to 'RETURN'");
    }

    #[test]
    fn keyword_completion_no_paren() {
        let h = ReplHelper::new();
        let (_, pairs) = h.complete_inner("od", 2);
        let od_pair = pairs.iter().find(|p| p.0 == "od").unwrap();
        assert_eq!(od_pair.1, "od", "keyword 'od' should complete without paren");
    }
}
