//! Evaluator core for the q-Kangaroo REPL.
//!
//! Walks [`AstNode`] trees, manages a variable [`Environment`], performs
//! arithmetic on [`Value`] types, catches panics from qsym-core, and
//! dispatches function calls.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use qsym_core::number::{QInt, QRat};
use qsym_core::qseries::{self, QMonomial, PochhammerOrder};
use qsym_core::qseries::{HypergeometricSeries, BilateralHypergeometricSeries};
use qsym_core::series::arithmetic;
use qsym_core::series::bivariate::{self as bv, BivariateSeries};
use qsym_core::series::trivariate::{self as tv, TrivariateSeries};
use qsym_core::series::FormalPowerSeries;
use qsym_core::symbol::SymbolId;

use crate::ast::{AstNode, BinOp, BoolBinOp, CompOp, Stmt, Terminator};
use crate::environment::Environment;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Sentinel truncation order for exact polynomials (no O(...) in display).
/// Chosen to be large enough to never interfere with real truncation orders,
/// but small enough to avoid overflow in min() comparisons.
pub(crate) const POLYNOMIAL_ORDER: i64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Procedure struct
// ---------------------------------------------------------------------------

/// A user-defined procedure (stored as AST, re-evaluated on each call).
#[derive(Clone, Debug)]
pub struct Procedure {
    /// Display name (set when assigned to a variable via `:=`).
    pub name: String,
    /// Formal parameter names.
    pub params: Vec<String>,
    /// Local variable names declared with `local`.
    pub locals: Vec<String>,
    /// Whether `option remember` was specified.
    pub remember: bool,
    /// Body statements (AST, re-evaluated on each call).
    pub body: Vec<Stmt>,
    /// Shared memoization table (keyed by Debug-string of args).
    pub memo: Rc<RefCell<HashMap<String, Value>>>,
}

// ---------------------------------------------------------------------------
// Value enum
// ---------------------------------------------------------------------------

/// A runtime value in the evaluator.
///
/// Unifies all possible return types from function calls and expressions.
#[derive(Clone, Debug)]
pub enum Value {
    /// Formal power series (most common return type).
    Series(FormalPowerSeries),
    /// Exact integer.
    Integer(QInt),
    /// Exact rational number.
    Rational(QRat),
    /// List of values.
    List(Vec<Value>),
    /// Key-value map (prodmake, etamake, qfactor results, etc.).
    Dict(Vec<(String, Value)>),
    /// Pair of values (heine transforms, bailey weak lemma).
    Pair(Box<Value>, Box<Value>),
    /// Boolean value.
    Bool(bool),
    /// String value (for filenames, etc.).
    String(String),
    /// None/null (try_summation returns None on failure).
    None,
    /// The infinity keyword.
    Infinity,
    /// A symbolic variable name (undefined name fallback, Maple-like).
    Symbol(String),
    /// Jacobi product expression: product of (q^a;q^b)_inf^exp factors.
    /// Each triple is (a, b, exponent). Maintained in canonical form.
    JacobiProduct(Vec<(i64, i64, i64)>),
    /// Q-product factorization: scalar * prod (1-q^i)^{mult_i}.
    /// Stores factors as BTreeMap<i64, i64> (i -> multiplicity).
    QProduct {
        factors: BTreeMap<i64, i64>,
        scalar: QRat,
        is_exact: bool,
    },
    /// Eta-quotient: prod eta(d*tau)^{r_d} * q^{q_shift}.
    /// Stores factors as BTreeMap<i64, i64> (d -> r_d).
    EtaQuotient {
        factors: BTreeMap<i64, i64>,
        q_shift: QRat,
    },
    /// User-defined procedure.
    Procedure(Procedure),
    /// Bivariate series: Laurent polynomial in outer variable with FPS coefficients.
    BivariateSeries(BivariateSeries),
    /// Trivariate series: Laurent polynomial in two outer variables with FPS coefficients.
    TrivariateSeries(TrivariateSeries),
    /// Fractional power series: inner FPS with exponent keys representing q^(k/denom).
    /// For example, q^(1/4) is stored as monomial at k=1 with denom=4.
    FractionalPowerSeries {
        inner: FormalPowerSeries,
        denom: i64,
    },
}

impl Value {
    /// Human-readable type name for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Series(_) => "series",
            Value::Integer(_) => "integer",
            Value::Rational(_) => "rational",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Pair(_, _) => "pair",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::None => "none",
            Value::Infinity => "infinity",
            Value::Symbol(_) => "symbol",
            Value::JacobiProduct(_) => "jacobi_product",
            Value::QProduct { .. } => "qproduct",
            Value::EtaQuotient { .. } => "eta_quotient",
            Value::Procedure(_) => "procedure",
            Value::BivariateSeries(_) => "bivariate_series",
            Value::TrivariateSeries(_) => "trivariate_series",
            Value::FractionalPowerSeries { .. } => "fractional_power_series",
        }
    }
}

// ---------------------------------------------------------------------------
// EvalError enum
// ---------------------------------------------------------------------------

/// An error produced during evaluation.
#[derive(Debug)]
pub enum EvalError {
    /// Reference to an undefined variable.
    UnknownVariable { name: String },
    /// Call to an unknown function (with suggested similar names).
    UnknownFunction { name: String, suggestions: Vec<String> },
    /// Wrong number of arguments.
    WrongArgCount {
        function: String,
        expected: String,
        got: usize,
        signature: String,
    },
    /// Wrong argument type.
    ArgType {
        function: String,
        arg_index: usize,
        expected: &'static str,
        got: String,
    },
    /// Type error in binary operation.
    TypeError {
        operation: String,
        left: String,
        right: String,
    },
    /// `%` reference with no previous result.
    NoLastResult,
    /// Caught panic from qsym-core.
    Panic(String),
    /// Other error.
    Other(String),
    /// Early return from a procedure body (RETURN(value)).
    /// If this propagates to top level, it means RETURN was used outside a procedure.
    EarlyReturn(Value),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UnknownVariable { name } => {
                write!(f, "Error: undefined variable '{}'", name)
            }
            EvalError::UnknownFunction { name, suggestions } => {
                if suggestions.is_empty() {
                    write!(f, "Error: unknown function '{}'", name)
                } else {
                    write!(
                        f,
                        "Error: unknown function '{}'. Did you mean: {}?",
                        name,
                        suggestions.join(", ")
                    )
                }
            }
            EvalError::WrongArgCount { function, expected, got, signature } => {
                write!(
                    f,
                    "Error: {} expects {} arguments ({}), got {}",
                    function, expected, signature, got
                )
            }
            EvalError::ArgType { function, arg_index, expected, got } => {
                write!(
                    f,
                    "Error: {} argument {} must be {}, got {}",
                    function,
                    arg_index + 1,
                    expected,
                    got
                )
            }
            EvalError::TypeError { operation, left, right } => {
                write!(
                    f,
                    "Error: cannot apply '{}' to {} and {}",
                    operation, left, right
                )
            }
            EvalError::NoLastResult => {
                write!(f, "Error: no previous result (use % after computing something)")
            }
            EvalError::Panic(msg) => {
                write!(f, "Error: computation failed: {}", msg)
            }
            EvalError::Other(msg) => {
                write!(f, "Error: {}", msg)
            }
            EvalError::EarlyReturn(_) => {
                write!(f, "Error: RETURN used outside of a procedure")
            }
        }
    }
}

impl std::error::Error for EvalError {}

// ---------------------------------------------------------------------------
// Argument extraction helpers
// ---------------------------------------------------------------------------

/// Check that exactly `expected` arguments were given.
pub fn expect_args(name: &str, args: &[Value], expected: usize) -> Result<(), EvalError> {
    if args.len() != expected {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: format!("{}", expected),
            got: args.len(),
            signature: get_signature(name),
        });
    }
    Ok(())
}

/// Check that the number of arguments is in `[min, max]`.
pub fn expect_args_range(
    name: &str,
    args: &[Value],
    min: usize,
    max: usize,
) -> Result<(), EvalError> {
    if args.len() < min || args.len() > max {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: if min == max {
                format!("{}", min)
            } else {
                format!("{}-{}", min, max)
            },
            got: args.len(),
            signature: get_signature(name),
        });
    }
    Ok(())
}

/// Extract an `i64` from args at `index`. Handles `Integer` (via QInt).
pub fn extract_i64(name: &str, args: &[Value], index: usize) -> Result<i64, EvalError> {
    match &args[index] {
        Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "integer (fits in i64)",
            got: "integer too large".to_string(),
        }),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "integer",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a `QRat` from args at `index`. Accepts Integer (promotes) or Rational.
pub fn extract_qrat(name: &str, args: &[Value], index: usize) -> Result<QRat, EvalError> {
    match &args[index] {
        Value::Integer(n) => Ok(QRat::from(n.clone())),
        Value::Rational(r) => Ok(r.clone()),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "number (integer or rational)",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a `FormalPowerSeries` from args at `index` (clones).
pub fn extract_series(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<FormalPowerSeries, EvalError> {
    match &args[index] {
        Value::Series(fps) => Ok(fps.clone()),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "series",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a list of `FormalPowerSeries` from args at `index`.
pub fn extract_series_list(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<Vec<FormalPowerSeries>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Series(fps) => result.push(fps.clone()),
                    other => {
                        return Err(EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: index,
                            expected: "list of series",
                            got: format!("list containing {} at position {}", other.type_name(), i),
                        });
                    }
                }
            }
            Ok(result)
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "list of series",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a `bool` from args at `index`.
pub fn extract_bool(name: &str, args: &[Value], index: usize) -> Result<bool, EvalError> {
    match &args[index] {
        Value::Bool(b) => Ok(*b),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "bool",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a list of i64 values from args at `index`.
pub fn extract_i64_list(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<Vec<i64>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Integer(n) => {
                        let v = n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: index,
                            expected: "list of integers (fits in i64)",
                            got: format!("integer too large at position {}", i),
                        })?;
                        result.push(v);
                    }
                    other => {
                        return Err(EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: index,
                            expected: "list of integers",
                            got: format!("list containing {} at position {}", other.type_name(), i),
                        });
                    }
                }
            }
            Ok(result)
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "list of integers",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a list of QRat values from args at `index`.
pub fn extract_qrat_list(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<Vec<QRat>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Integer(n) => result.push(QRat::from(n.clone())),
                    Value::Rational(r) => result.push(r.clone()),
                    other => {
                        return Err(EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: index,
                            expected: "list of numbers",
                            got: format!("list containing {} at position {}", other.type_name(), i),
                        });
                    }
                }
            }
            Ok(result)
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "list of numbers",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a list of QMonomials from args at `index`.
///
/// Each element must be a `Value::List` of exactly 3 integers: [num, den, power].
/// Builds `QMonomial::new(QRat::from((num, den)), power)` for each.
pub fn extract_monomial_list(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<Vec<QMonomial>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::List(inner) if inner.len() == 3 => {
                        let sub = inner;
                        let num = match &sub[0] {
                            Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: index,
                                expected: "list of [num, den, power] triples",
                                got: format!("integer too large in triple {}", i),
                            })?,
                            other => {
                                return Err(EvalError::ArgType {
                                    function: name.to_string(),
                                    arg_index: index,
                                    expected: "list of [num, den, power] triples",
                                    got: format!("{} in triple {} position 0", other.type_name(), i),
                                });
                            }
                        };
                        let den = match &sub[1] {
                            Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: index,
                                expected: "list of [num, den, power] triples",
                                got: format!("integer too large in triple {}", i),
                            })?,
                            other => {
                                return Err(EvalError::ArgType {
                                    function: name.to_string(),
                                    arg_index: index,
                                    expected: "list of [num, den, power] triples",
                                    got: format!("{} in triple {} position 1", other.type_name(), i),
                                });
                            }
                        };
                        let power = match &sub[2] {
                            Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: index,
                                expected: "list of [num, den, power] triples",
                                got: format!("integer too large in triple {}", i),
                            })?,
                            other => {
                                return Err(EvalError::ArgType {
                                    function: name.to_string(),
                                    arg_index: index,
                                    expected: "list of [num, den, power] triples",
                                    got: format!("{} in triple {} position 2", other.type_name(), i),
                                });
                            }
                        };
                        result.push(QMonomial::new(QRat::from((num, den)), power));
                    }
                    other => {
                        return Err(EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: index,
                            expected: "list of [num, den, power] triples",
                            got: format!("{} at position {} (expected 3-element list)", other.type_name(), i),
                        });
                    }
                }
            }
            Ok(result)
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "list of [num, den, power] triples",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a SymbolId from a Value::Symbol, interning it in the registry.
fn extract_symbol_id(
    name: &str,
    args: &[Value],
    index: usize,
    env: &mut Environment,
) -> Result<SymbolId, EvalError> {
    match &args[index] {
        Value::Symbol(s) => Ok(env.symbols.intern(s)),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "symbol (variable name)",
            got: other.type_name().to_string(),
        }),
    }
}

/// Extract a list of Symbol values as `Vec<String>` (for SL label lists).
fn extract_symbol_list(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<Vec<String>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut labels = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Symbol(s) => labels.push(s.clone()),
                    other => {
                        return Err(EvalError::Other(format!(
                            "{}: Argument {} (SL): element {} must be a symbol, got {}",
                            name,
                            index + 1,
                            i + 1,
                            other.type_name()
                        )));
                    }
                }
            }
            Ok(labels)
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "list of symbols",
            got: other.type_name().to_string(),
        }),
    }
}

/// Check for duplicate labels in an SL list.
fn validate_unique_labels(name: &str, labels: &[String]) -> Result<(), EvalError> {
    let mut seen = HashSet::new();
    for label in labels {
        if !seen.insert(label.as_str()) {
            return Err(EvalError::Other(format!(
                "{}: duplicate label '{}' in SL",
                name, label
            )));
        }
    }
    Ok(())
}

/// Simple trial division primality test.
fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5i64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Format JacobiProduct factors as display string for qs2jaccombo output.
fn format_jacobi_product_value(factors: &[(i64, i64, i64)]) -> String {
    if factors.is_empty() {
        return "1".to_string();
    }
    let parts: Vec<String> = factors.iter().map(|&(a, b, exp)| {
        if exp == 1 {
            format!("JAC({},{})", a, b)
        } else {
            format!("JAC({},{})^({})", a, b, exp)
        }
    }).collect();
    parts.join("*")
}

/// Format a linear combination with symbolic labels.
///
/// For coefficients `[c1, c2, ...]` and labels `[F1, F2, ...]`, produces
/// strings like `"12*F1 + 13*F2"`. Handles coefficient 1 (just label),
/// -1 (just -label), 0 (skip). Replaces `"+ -"` with `"- "`.
fn format_linear_combo(coeffs: &[QRat], labels: &[String]) -> String {
    let mut parts = Vec::new();
    for (c, label) in coeffs.iter().zip(labels.iter()) {
        if c.is_zero() {
            continue;
        }
        let one = QRat::from((1i64, 1i64));
        let neg_one = QRat::from((-1i64, 1i64));
        let coeff_str = if *c == one {
            label.clone()
        } else if *c == neg_one {
            format!("-{}", label)
        } else {
            format!("{}*{}", c, label)
        };
        parts.push(coeff_str);
    }
    if parts.is_empty() {
        "0".to_string()
    } else {
        parts.join(" + ").replace("+ -", "- ")
    }
}

/// Format a linear combination with i64 coefficients mod p.
fn format_linear_combo_modp(coeffs: &[i64], labels: &[String], _p: i64) -> String {
    let mut parts = Vec::new();
    for (&c, label) in coeffs.iter().zip(labels.iter()) {
        if c == 0 {
            continue;
        }
        let coeff_str = if c == 1 {
            label.clone()
        } else if c == -1 {
            format!("-{}", label)
        } else {
            format!("{}*{}", c, label)
        };
        parts.push(coeff_str);
    }
    if parts.is_empty() {
        "0".to_string()
    } else {
        parts.join(" + ").replace("+ -", "- ")
    }
}

/// Format a polynomial expression from a null-space vector and monomial list.
///
/// For each nonzero coefficient, builds a monomial string (e.g., `X[1]^2*X[2]`)
/// and combines with the coefficient.
fn format_polynomial_expr(coeffs: &[QRat], monomials: &[Vec<i64>], labels: &[String]) -> String {
    let mut parts = Vec::new();
    for (c, mono) in coeffs.iter().zip(monomials.iter()) {
        if c.is_zero() {
            continue;
        }
        // Build monomial string
        let mut mono_parts = Vec::new();
        for (i, &e) in mono.iter().enumerate() {
            if e == 0 {
                continue;
            }
            if e == 1 {
                mono_parts.push(labels[i].clone());
            } else {
                mono_parts.push(format!("{}^{}", labels[i], e));
            }
        }
        let monomial_str = if mono_parts.is_empty() {
            "1".to_string()
        } else {
            mono_parts.join("*")
        };

        let one = QRat::from((1i64, 1i64));
        let neg_one = QRat::from((-1i64, 1i64));
        let term = if *c == one {
            monomial_str
        } else if *c == neg_one {
            format!("-{}", monomial_str)
        } else if mono_parts.is_empty() {
            // Just a constant term
            format!("{}", c)
        } else {
            format!("{}*{}", c, monomial_str)
        };
        parts.push(term);
    }
    if parts.is_empty() {
        "0".to_string()
    } else {
        parts.join(" + ").replace("+ -", "- ")
    }
}

/// Format a polynomial expression with i64 coefficients mod p.
fn format_polynomial_expr_modp(coeffs: &[i64], monomials: &[Vec<i64>], labels: &[String], _p: i64) -> String {
    let mut parts = Vec::new();
    for (&c, mono) in coeffs.iter().zip(monomials.iter()) {
        if c == 0 {
            continue;
        }
        let mut mono_parts = Vec::new();
        for (i, &e) in mono.iter().enumerate() {
            if e == 0 {
                continue;
            }
            if e == 1 {
                mono_parts.push(labels[i].clone());
            } else {
                mono_parts.push(format!("{}^{}", labels[i], e));
            }
        }
        let monomial_str = if mono_parts.is_empty() {
            "1".to_string()
        } else {
            mono_parts.join("*")
        };

        let term = if c == 1 {
            monomial_str
        } else if c == -1 {
            format!("-{}", monomial_str)
        } else if mono_parts.is_empty() {
            format!("{}", c)
        } else {
            format!("{}*{}", c, monomial_str)
        };
        parts.push(term);
    }
    if parts.is_empty() {
        "0".to_string()
    } else {
        parts.join(" + ").replace("+ -", "- ")
    }
}

/// Format a `PolynomialRelation` as a polynomial in X, Y variables.
fn format_findpoly_result(rel: &qseries::PolynomialRelation) -> String {
    let mut parts = Vec::new();
    for (i, row) in rel.coefficients.iter().enumerate() {
        for (j, c) in row.iter().enumerate() {
            if c.is_zero() {
                continue;
            }
            // Build variable part
            let x_part = match i {
                0 => String::new(),
                1 => "X".to_string(),
                _ => format!("X^{}", i),
            };
            let y_part = match j {
                0 => String::new(),
                1 => "Y".to_string(),
                _ => format!("Y^{}", j),
            };
            let var_str = match (x_part.is_empty(), y_part.is_empty()) {
                (true, true) => String::new(),   // constant term
                (false, true) => x_part,
                (true, false) => y_part,
                (false, false) => format!("{}*{}", x_part, y_part),
            };

            let one = QRat::from((1i64, 1i64));
            let neg_one = QRat::from((-1i64, 1i64));
            let term = if var_str.is_empty() {
                format!("{}", c)
            } else if *c == one {
                var_str
            } else if *c == neg_one {
                format!("-{}", var_str)
            } else {
                format!("{}*{}", c, var_str)
            };
            parts.push(term);
        }
    }
    if parts.is_empty() {
        "0".to_string()
    } else {
        parts.join(" + ").replace("+ -", "- ")
    }
}

/// Verify a findpoly result by evaluating P(x, y) and checking it equals zero
/// to O(q^check_order).
fn verify_findpoly_result(
    rel: &qseries::PolynomialRelation,
    x: &FormalPowerSeries,
    y: &FormalPowerSeries,
    check_order: i64,
) -> bool {
    let var = x.variable();
    let trunc = check_order.min(x.truncation_order()).min(y.truncation_order());

    // Truncate inputs to check_order
    let x_trunc = truncate_fps(x, trunc);
    let y_trunc = truncate_fps(y, trunc);

    let mut total = FormalPowerSeries::zero(var, trunc);

    for (i, row) in rel.coefficients.iter().enumerate() {
        for (j, c) in row.iter().enumerate() {
            if c.is_zero() {
                continue;
            }
            // Compute x^i * y^j
            let mut term = FormalPowerSeries::one(var, trunc);
            for _ in 0..i {
                term = fps_mul_truncated(&term, &x_trunc, trunc);
            }
            for _ in 0..j {
                term = fps_mul_truncated(&term, &y_trunc, trunc);
            }
            // Scale by coefficient c
            let scaled = fps_scale(&term, c);
            total = fps_add(&total, &scaled);
        }
    }

    // Verify all coefficients below check_order are zero
    for (&k, v) in total.iter() {
        if k < trunc && !v.is_zero() {
            return false;
        }
    }
    true
}

/// Truncate an FPS to the given order.
fn truncate_fps(f: &FormalPowerSeries, trunc: i64) -> FormalPowerSeries {
    let mut coeffs = std::collections::BTreeMap::new();
    for (&k, v) in f.iter() {
        if k < trunc {
            coeffs.insert(k, v.clone());
        }
    }
    FormalPowerSeries::from_coeffs(f.variable(), coeffs, trunc)
}

/// Multiply two FPS and truncate to the given order.
fn fps_mul_truncated(a: &FormalPowerSeries, b: &FormalPowerSeries, trunc: i64) -> FormalPowerSeries {
    let mut coeffs = std::collections::BTreeMap::new();
    for (&ka, va) in a.iter() {
        for (&kb, vb) in b.iter() {
            let k = ka + kb;
            if k < trunc {
                let prod = va.clone() * vb.clone();
                let entry = coeffs.entry(k).or_insert_with(QRat::zero);
                *entry = entry.clone() + prod;
            }
        }
    }
    coeffs.retain(|_, v| !v.is_zero());
    FormalPowerSeries::from_coeffs(a.variable(), coeffs, trunc)
}

/// Scale all coefficients of an FPS by a rational.
fn fps_scale(f: &FormalPowerSeries, s: &QRat) -> FormalPowerSeries {
    let mut coeffs = std::collections::BTreeMap::new();
    for (&k, v) in f.iter() {
        let scaled = v.clone() * s.clone();
        if !scaled.is_zero() {
            coeffs.insert(k, scaled);
        }
    }
    FormalPowerSeries::from_coeffs(f.variable(), coeffs, f.truncation_order())
}

/// Add two FPS.
fn fps_add(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    let trunc = a.truncation_order().min(b.truncation_order());
    let mut coeffs = std::collections::BTreeMap::new();
    for (&k, v) in a.iter() {
        if k < trunc {
            coeffs.insert(k, v.clone());
        }
    }
    for (&k, v) in b.iter() {
        if k < trunc {
            let entry = coeffs.entry(k).or_insert_with(QRat::zero);
            *entry = entry.clone() + v.clone();
        }
    }
    coeffs.retain(|_, v| !v.is_zero());
    FormalPowerSeries::from_coeffs(a.variable(), coeffs, trunc)
}

/// Format a ZQFactorization result as a human-readable product string.
fn format_zqfactor_result(zqf: &qseries::ZQFactorization, z_var: &str) -> String {
    if zqf.scalar.is_zero() {
        return "0".to_string();
    }

    let mut parts = Vec::new();

    if zqf.scalar != QRat::one() {
        parts.push(format!("{}", zqf.scalar));
    }

    for &(z_pow, q_pow, mult) in &zqf.factors {
        let factor_str = match (z_pow, q_pow) {
            (1, 0) => format!("(1-{})", z_var),
            (1, p) => format!("(1-{}*q^{})", z_var, p),
            (-1, p) => format!("(1-q^{}/{})", p, z_var),
            (zp, 0) => format!("(1-{}^{})", z_var, zp),
            (zp, qp) if zp > 0 => format!("(1-{}^{}*q^{})", z_var, zp, qp),
            (zp, qp) => format!("(1-q^{}/{}^{})", qp, z_var, -zp),
        };
        if mult == 1 {
            parts.push(factor_str);
        } else {
            parts.push(format!("{}^{}", factor_str, mult));
        }
    }

    for (&i, &mult) in &zqf.q_factors {
        let factor_str = format!("(1-q^{})", i);
        if mult == 1 {
            parts.push(factor_str);
        } else if mult > 0 {
            parts.push(format!("{}^{}", factor_str, mult));
        } else {
            parts.push(format!("1/{}^{}", factor_str, -mult));
        }
    }

    if parts.is_empty() {
        "1".to_string()
    } else {
        parts.join("*")
    }
}

/// Generate default labels X[1], X[2], ..., X[k] matching Garvan's convention.
fn default_labels(k: usize) -> Vec<String> {
    (1..=k).map(|i| format!("X[{}]", i)).collect()
}

/// Extract a QMonomial from an argument that is a Symbol (var^1) or a Series monomial.
fn extract_monomial_from_arg(
    func_name: &str,
    args: &[Value],
    index: usize,
) -> Result<QMonomial, EvalError> {
    match &args[index] {
        Value::Symbol(_) => {
            // Symbol = var^1, coefficient 1
            Ok(QMonomial::new(QRat::one(), 1))
        }
        Value::Series(fps) => {
            // Extract monomial from single-term series
            let terms: Vec<_> = fps.iter().collect();
            if terms.len() == 1 {
                let (&power, coeff) = terms[0];
                Ok(QMonomial::new(coeff.clone(), power))
            } else if terms.is_empty() {
                Ok(QMonomial::new(QRat::zero(), 0))
            } else {
                Err(EvalError::ArgType {
                    function: func_name.to_string(),
                    arg_index: index,
                    expected: "monomial (single-term series like q^2)",
                    got: format!("polynomial with {} terms", terms.len()),
                })
            }
        }
        Value::Integer(n) => {
            Ok(QMonomial::new(QRat::from(n.clone()), 0))
        }
        other => Err(EvalError::ArgType {
            function: func_name.to_string(),
            arg_index: index,
            expected: "monomial expression (e.g., q^2) or symbol",
            got: other.type_name().to_string(),
        }),
    }
}

/// Build a `HypergeometricSeries` from standard 6-arg pattern:
/// (upper_list, lower_list, z_num, z_den, z_pow, order).
///
/// Returns the series struct and the truncation order.
fn build_hypergeometric(
    name: &str,
    args: &[Value],
) -> Result<(HypergeometricSeries, i64), EvalError> {
    expect_args(name, args, 6)?;
    let upper = extract_monomial_list(name, args, 0)?;
    let lower = extract_monomial_list(name, args, 1)?;
    let z_num = extract_i64(name, args, 2)?;
    let z_den = extract_i64(name, args, 3)?;
    let z_pow = extract_i64(name, args, 4)?;
    let order = extract_i64(name, args, 5)?;
    let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);
    let series = HypergeometricSeries {
        upper,
        lower,
        argument: z,
    };
    Ok((series, order))
}

/// Build a `BilateralHypergeometricSeries` from standard 6-arg pattern.
fn build_bilateral(
    name: &str,
    args: &[Value],
) -> Result<(BilateralHypergeometricSeries, i64), EvalError> {
    expect_args(name, args, 6)?;
    let upper = extract_monomial_list(name, args, 0)?;
    let lower = extract_monomial_list(name, args, 1)?;
    let z_num = extract_i64(name, args, 2)?;
    let z_den = extract_i64(name, args, 3)?;
    let z_pow = extract_i64(name, args, 4)?;
    let order = extract_i64(name, args, 5)?;
    let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);
    let series = BilateralHypergeometricSeries {
        upper,
        lower,
        argument: z,
    };
    Ok((series, order))
}

/// Build a `QMonomial` from 3 consecutive args at offset.
fn extract_monomial(
    name: &str,
    args: &[Value],
    offset: usize,
) -> Result<QMonomial, EvalError> {
    let num = extract_i64(name, args, offset)?;
    let den = extract_i64(name, args, offset + 1)?;
    let power = extract_i64(name, args, offset + 2)?;
    Ok(QMonomial::new(QRat::from((num, den)), power))
}

// ---------------------------------------------------------------------------
// Statement evaluation
// ---------------------------------------------------------------------------

/// Evaluate a statement, returning `Some(value)` if the result should be
/// printed, or `None` if suppressed (colon terminator).
pub fn eval_stmt(stmt: &Stmt, env: &mut Environment) -> Result<Option<Value>, EvalError> {
    let value = eval_expr(&stmt.node, env)?;

    // Store last result (for `%` reference)
    env.last_result = Some(value.clone());

    // Respect terminator: Semi/Implicit -> show, Colon -> suppress
    match stmt.terminator {
        Terminator::Semi | Terminator::Implicit => Ok(Some(value)),
        Terminator::Colon => Ok(None),
    }
}

/// Translate common qsym-core panic messages to human-readable text.
///
/// Uses `contains()` for robustness against minor wording changes.
/// Falls back to the raw message if no translation matches.
fn translate_panic_message(raw: &str) -> String {
    if raw.contains("Cannot invert series with zero constant term") {
        return "cannot invert a series whose constant term is zero (the series \
                starts at q^k with k > 0; try shifting or extracting the leading \
                power first)"
            .to_string();
    }
    if raw.contains("division by zero") || raw.contains("Division by zero") {
        return "division by zero".to_string();
    }
    if raw.contains("Cannot invert zero") {
        return "cannot invert zero".to_string();
    }
    if raw.contains("index out of bounds") {
        return "index out of bounds".to_string();
    }
    // Strip "thread '<name>' panicked at '<msg>'" prefix if present
    // (shouldn't happen with catch_unwind, but defensive)
    if raw.contains("panicked at") {
        // Format: "thread 'main' panicked at 'actual message', file:line:col"
        // or: "thread 'main' panicked at actual message"
        if let Some(at_pos) = raw.find("panicked at ") {
            let after = &raw[at_pos + "panicked at ".len()..];
            // Strip surrounding quotes if present
            let msg = after.trim_start_matches('\'').trim_end_matches('\'');
            // Strip trailing ", file:line:col" if present
            let msg = if let Some(comma_pos) = msg.rfind(", ") {
                &msg[..comma_pos]
            } else {
                msg
            };
            // Recursively translate the extracted message
            return translate_panic_message(msg);
        }
    }
    raw.to_string()
}

/// Evaluate a statement with panic catching.
///
/// Wraps [`eval_stmt`] in `catch_unwind` with `AssertUnwindSafe`.
/// On panic, extracts the message and returns `EvalError::Panic`.
/// Panic messages are translated to human-friendly text via
/// [`translate_panic_message`].
///
/// `AssertUnwindSafe` is safe here because after a panic the environment's
/// variables may have partial updates but the rug heap is not corrupted.
/// Each statement either succeeds completely or the result is discarded.
pub fn eval_stmt_safe(stmt: &Stmt, env: &mut Environment) -> Result<Option<Value>, EvalError> {
    let result = catch_unwind(AssertUnwindSafe(|| eval_stmt(stmt, env)));

    match result {
        Ok(inner) => inner,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "internal computation error".to_string()
            };
            Err(EvalError::Panic(translate_panic_message(&msg)))
        }
    }
}

// ---------------------------------------------------------------------------
// Expression evaluation
// ---------------------------------------------------------------------------

/// Recursively evaluate an AST node to a [`Value`].
pub fn eval_expr(node: &AstNode, env: &mut Environment) -> Result<Value, EvalError> {
    match node {
        AstNode::Integer(n) => Ok(Value::Integer(QInt::from(*n))),

        AstNode::BigInteger(s) => {
            let int = rug::Integer::from_str_radix(s, 10)
                .map_err(|e| EvalError::Other(format!("invalid big integer '{}': {}", s, e)))?;
            Ok(Value::Integer(QInt(int)))
        }

        AstNode::Infinity => Ok(Value::Infinity),

        AstNode::StringLit(s) => Ok(Value::String(s.clone())),

        AstNode::LastResult => match &env.last_result {
            Some(val) => Ok(val.clone()),
            None => Err(EvalError::NoLastResult),
        },

        AstNode::Variable(name) => match env.get_var(name) {
            Some(val) => Ok(val.clone()),
            None => Ok(Value::Symbol(name.clone())),
        },

        AstNode::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                result.push(eval_expr(item, env)?);
            }
            Ok(Value::List(result))
        }

        AstNode::Index { expr, index } => {
            let base = eval_expr(expr, env)?;
            let idx_val = eval_expr(index, env)?;
            let i = match &idx_val {
                Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::Other(
                    "index too large".to_string()
                ))?,
                _ => return Err(EvalError::Other(
                    format!("index must be an integer, got {}", idx_val.type_name())
                )),
            };
            match base {
                Value::List(items) => {
                    // Maple uses 1-indexing
                    if i < 1 || i as usize > items.len() {
                        return Err(EvalError::Other(format!(
                            "list index {} out of range (list has {} elements)",
                            i, items.len()
                        )));
                    }
                    Ok(items[(i - 1) as usize].clone())
                }
                Value::Symbol(ref name) => {
                    // Backward compat: if X is unbound (became Symbol), fall back to
                    // looking up "X[i]" as a variable name (table-style indexed variables)
                    let key = format!("{}[{}]", name, i);
                    match env.get_var(&key) {
                        Some(val) => Ok(val.clone()),
                        None => Err(EvalError::Other(format!(
                            "cannot index into symbol '{}' (not a list, and '{}' is not defined)",
                            name, key
                        ))),
                    }
                }
                _ => Err(EvalError::Other(format!(
                    "cannot index into {}", base.type_name()
                ))),
            }
        }

        AstNode::IndexAssign { name, index, value } => {
            let idx_val = eval_expr(index, env)?;
            let i = match &idx_val {
                Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::Other(
                    "index too large".to_string()
                ))?,
                _ => return Err(EvalError::Other(
                    format!("index must be an integer, got {}", idx_val.type_name())
                )),
            };
            let val = eval_expr(value, env)?;

            // Check if name is currently a list
            if let Some(Value::List(ref items)) = env.get_var(name) {
                // Mutate list element in place
                if i < 1 || i as usize > items.len() {
                    return Err(EvalError::Other(format!(
                        "list index {} out of range (list has {} elements)",
                        i, items.len()
                    )));
                }
                let mut new_items = items.clone();
                new_items[(i - 1) as usize] = val.clone();
                env.set_var(name, Value::List(new_items));
            } else {
                // Fall back to table-style: set "name[i]" as a variable
                let key = format!("{}[{}]", name, i);
                env.set_var(&key, val.clone());
            }
            Ok(val)
        }

        AstNode::Neg(inner) => {
            let val = eval_expr(inner, env)?;
            eval_negate(val, env)
        }

        AstNode::BinOp { op, lhs, rhs } => {
            let left = eval_expr(lhs, env)?;
            let right = eval_expr(rhs, env)?;
            eval_binop(*op, left, right, env)
        }

        AstNode::FuncCall { name, args } => {
            // Special-case: RETURN(value) produces EarlyReturn error
            if name == "RETURN" {
                if args.len() != 1 {
                    return Err(EvalError::WrongArgCount {
                        function: "RETURN".to_string(),
                        expected: "1".to_string(),
                        got: args.len(),
                        signature: "RETURN(value)".to_string(),
                    });
                }
                let val = eval_expr(&args[0], env)?;
                return Err(EvalError::EarlyReturn(val));
            }

            // Special-case: subs(var=val, ..., expr) with AST-level interception
            // Each substitution arg is parsed as AstNode::Compare(Eq), which we intercept
            // before evaluation to avoid q=1 becoming Bool.
            if name == "subs" {
                if args.len() < 2 {
                    return Err(EvalError::WrongArgCount {
                        function: "subs".to_string(),
                        expected: "at least 2".to_string(),
                        got: args.len(),
                        signature: "subs(var=val, ..., expr)".to_string(),
                    });
                }
                // Evaluate target (last argument)
                let mut target = eval_expr(&args[args.len() - 1], env)?;
                // Process each substitution pair (all args except the last)
                for i in 0..(args.len() - 1) {
                    match &args[i] {
                        AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
                            let var_name = match lhs.as_ref() {
                                AstNode::Variable(vname) => vname.clone(),
                                AstNode::Index { expr, index } => {
                                    // Handle indexed variables: X[1]=val -> var_name "X[1]"
                                    let base_name = match expr.as_ref() {
                                        AstNode::Variable(n) => n.clone(),
                                        _ => return Err(EvalError::Other(
                                            "subs: indexed left side must be name[index]".into()
                                        )),
                                    };
                                    let idx = match index.as_ref() {
                                        AstNode::Integer(n) => *n,
                                        _ => return Err(EvalError::Other(
                                            "subs: index must be an integer".into()
                                        )),
                                    };
                                    format!("{}[{}]", base_name, idx)
                                }
                                _ => return Err(EvalError::Other(
                                    "subs: left side of = must be a variable name or name[index]".into()
                                )),
                            };
                            let sub_value = eval_expr(rhs, env)?;
                            target = perform_substitution(&var_name, sub_value, target, env)?;
                        }
                        _ => return Err(EvalError::Other(
                            "subs: each substitution must be var=value".into()
                        )),
                    }
                }
                return Ok(target);
            }

            // Special-case: print(expr, ...) displays intermediate results
            if name == "print" {
                if args.is_empty() {
                    return Err(EvalError::WrongArgCount {
                        function: "print".to_string(),
                        expected: "at least 1".to_string(),
                        got: 0,
                        signature: "print(expr, ...)".to_string(),
                    });
                }
                let mut last_val = Value::None;
                for arg in args {
                    let val = eval_expr(arg, env)?;
                    println!("{}", crate::format::format_value(&val, &env.symbols));
                    last_val = val;
                }
                return Ok(last_val);
            }

            // Special-case: add/mul/seq(expr, var=a..b) with AST-level interception
            // Body expression must NOT be eagerly evaluated -- iterate with variable substitution.
            if name == "add" || name == "mul" || name == "seq" {
                return eval_iteration_func(name, args, env);
            }

            // Check if name refers to a user-defined procedure
            if let Some(Value::Procedure(proc_val)) = env.get_var(name).cloned() {
                let mut evaluated = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated.push(eval_expr(arg, env)?);
                }
                return call_procedure(&proc_val, &evaluated, env);
            }

            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(eval_expr(arg, env)?);
            }
            dispatch(name, &evaluated, env)
        }

        AstNode::Assign { name, value } => {
            // Check for Maple unassign syntax: x := 'x'
            // After parsing, 'x' becomes AstNode::StringLit("x")
            if let AstNode::StringLit(s) = value.as_ref() {
                if s == name {
                    env.variables.remove(name);
                    return Ok(Value::Symbol(name.clone()));
                }
            }
            let val = eval_expr(value, env)?;
            // Set procedure name when assigned to a variable
            let val = if let Value::Procedure(mut proc_val) = val {
                proc_val.name = name.clone();
                Value::Procedure(proc_val)
            } else {
                val
            };
            env.set_var(name, val.clone());
            Ok(val)
        }

        AstNode::Compare { op, lhs, rhs } => {
            let left = eval_expr(lhs, env)?;
            let right = eval_expr(rhs, env)?;
            eval_compare(*op, left, right)
        }

        AstNode::Not(inner) => {
            let val = eval_expr(inner, env)?;
            match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                other => Err(EvalError::Other(format!(
                    "operand of 'not' must be bool, got {}",
                    other.type_name()
                ))),
            }
        }

        AstNode::BoolOp { op, lhs, rhs } => {
            eval_bool_op(*op, lhs, rhs, env)
        }

        AstNode::ForLoop { var, from, to, by, body } => {
            eval_for_loop(var, from, to, by.as_deref(), body, env)
        }

        AstNode::WhileLoop { condition, body } => {
            eval_while_loop(condition, body, env)
        }

        AstNode::IfExpr { condition, then_body, elif_branches, else_body } => {
            eval_if_expr(condition, then_body, elif_branches, else_body.as_deref(), env)
        }

        AstNode::ProcDef { params, locals, options, body } => {
            let remember = options.iter().any(|o| o == "remember");
            Ok(Value::Procedure(Procedure {
                name: String::new(),
                params: params.clone(),
                locals: locals.clone(),
                remember,
                body: body.clone(),
                memo: Rc::new(RefCell::new(HashMap::new())),
            }))
        }

        AstNode::Lambda { param, body } => {
            Ok(Value::Procedure(Procedure {
                name: String::new(),
                params: vec![param.clone()],
                locals: vec![],
                remember: false,
                body: vec![Stmt {
                    node: body.as_ref().clone(),
                    terminator: Terminator::Implicit,
                }],
                memo: Rc::new(RefCell::new(HashMap::new())),
            }))
        }

        AstNode::Range { .. } => {
            Err(EvalError::Other(
                "range expressions (a..b) are only valid inside add(), mul(), or seq()".to_string()
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Control flow evaluation
// ---------------------------------------------------------------------------

/// Test whether a value is truthy (for use in conditions).
///
/// - Bool: direct boolean value
/// - Integer: nonzero is true (Maple convention)
/// - Other types: error
fn is_truthy(val: &Value) -> Result<bool, EvalError> {
    match val {
        Value::Bool(b) => Ok(*b),
        Value::Integer(n) => Ok(!n.is_zero()),
        Value::Symbol(s) => match s.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(EvalError::Other(format!(
                "expected boolean or integer in condition, got symbol '{}'",
                s
            ))),
        },
        other => Err(EvalError::Other(format!(
            "expected boolean or integer in condition, got {}",
            other.type_name()
        ))),
    }
}

/// Evaluate a comparison expression.
///
/// Supports Integer, Rational, Symbol (equality only), and Bool (equality only).
/// Mixed Integer/Rational comparisons promote the Integer to Rational.
fn eval_compare(op: CompOp, left: Value, right: Value) -> Result<Value, EvalError> {
    match (&left, &right) {
        // Integer vs Integer
        (Value::Integer(a), Value::Integer(b)) => {
            let result = match op {
                CompOp::Eq => a == b,
                CompOp::NotEq => a != b,
                CompOp::Less => a < b,
                CompOp::Greater => a > b,
                CompOp::LessEq => a <= b,
                CompOp::GreaterEq => a >= b,
            };
            Ok(Value::Bool(result))
        }
        // Rational vs Rational
        (Value::Rational(a), Value::Rational(b)) => {
            let result = match op {
                CompOp::Eq => a == b,
                CompOp::NotEq => a != b,
                CompOp::Less => a < b,
                CompOp::Greater => a > b,
                CompOp::LessEq => a <= b,
                CompOp::GreaterEq => a >= b,
            };
            Ok(Value::Bool(result))
        }
        // Integer vs Rational (promote Integer to Rational)
        (Value::Integer(a), Value::Rational(b)) => {
            let a_rat = QRat::from(a.clone());
            let result = match op {
                CompOp::Eq => a_rat == *b,
                CompOp::NotEq => a_rat != *b,
                CompOp::Less => a_rat < *b,
                CompOp::Greater => a_rat > *b,
                CompOp::LessEq => a_rat <= *b,
                CompOp::GreaterEq => a_rat >= *b,
            };
            Ok(Value::Bool(result))
        }
        // Rational vs Integer (promote Integer to Rational)
        (Value::Rational(a), Value::Integer(b)) => {
            let b_rat = QRat::from(b.clone());
            let result = match op {
                CompOp::Eq => *a == b_rat,
                CompOp::NotEq => *a != b_rat,
                CompOp::Less => *a < b_rat,
                CompOp::Greater => *a > b_rat,
                CompOp::LessEq => *a <= b_rat,
                CompOp::GreaterEq => *a >= b_rat,
            };
            Ok(Value::Bool(result))
        }
        // Symbol vs Symbol (equality only)
        (Value::Symbol(a), Value::Symbol(b)) => {
            match op {
                CompOp::Eq => Ok(Value::Bool(a == b)),
                CompOp::NotEq => Ok(Value::Bool(a != b)),
                _ => Err(EvalError::TypeError {
                    operation: format!("{}", match op {
                        CompOp::Less => "<",
                        CompOp::Greater => ">",
                        CompOp::LessEq => "<=",
                        CompOp::GreaterEq => ">=",
                        _ => unreachable!(),
                    }),
                    left: "symbol".to_string(),
                    right: "symbol".to_string(),
                }),
            }
        }
        // Bool vs Bool (equality only)
        (Value::Bool(a), Value::Bool(b)) => {
            match op {
                CompOp::Eq => Ok(Value::Bool(a == b)),
                CompOp::NotEq => Ok(Value::Bool(a != b)),
                _ => Err(EvalError::TypeError {
                    operation: format!("{}", match op {
                        CompOp::Less => "<",
                        CompOp::Greater => ">",
                        CompOp::LessEq => "<=",
                        CompOp::GreaterEq => ">=",
                        _ => unreachable!(),
                    }),
                    left: "bool".to_string(),
                    right: "bool".to_string(),
                }),
            }
        }
        // All other combinations
        _ => Err(EvalError::TypeError {
            operation: "comparison".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

/// Evaluate a short-circuit boolean operation.
///
/// Takes AST nodes (not Values) so that the right-hand side is only evaluated
/// when needed.
fn eval_bool_op(
    op: BoolBinOp,
    lhs: &AstNode,
    rhs: &AstNode,
    env: &mut Environment,
) -> Result<Value, EvalError> {
    let left_val = eval_expr(lhs, env)?;
    match op {
        BoolBinOp::And => {
            match &left_val {
                Value::Bool(false) => Ok(Value::Bool(false)),
                Value::Bool(true) => {
                    let right_val = eval_expr(rhs, env)?;
                    match &right_val {
                        Value::Bool(_) => Ok(right_val),
                        _ => Err(EvalError::Other(format!(
                            "operand of 'and' must be bool, got {}",
                            right_val.type_name()
                        ))),
                    }
                }
                _ => Err(EvalError::Other(format!(
                    "operand of 'and' must be bool, got {}",
                    left_val.type_name()
                ))),
            }
        }
        BoolBinOp::Or => {
            match &left_val {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => {
                    let right_val = eval_expr(rhs, env)?;
                    match &right_val {
                        Value::Bool(_) => Ok(right_val),
                        _ => Err(EvalError::Other(format!(
                            "operand of 'or' must be bool, got {}",
                            right_val.type_name()
                        ))),
                    }
                }
                _ => Err(EvalError::Other(format!(
                    "operand of 'or' must be bool, got {}",
                    left_val.type_name()
                ))),
            }
        }
    }
}

/// Evaluate a sequence of statements, returning the value of the last one.
///
/// Propagates EarlyReturn errors upward (only procedures catch them).
fn eval_stmt_sequence(stmts: &[Stmt], env: &mut Environment) -> Result<Value, EvalError> {
    let mut result = Value::None;
    for stmt in stmts {
        result = eval_expr(&stmt.node, env)?;
    }
    Ok(result)
}

/// Call a user-defined procedure with the given arguments.
///
/// Implements local variable scoping via save/restore, catches EarlyReturn
/// at the procedure boundary, and handles memoization for `option remember`.
fn call_procedure(proc: &Procedure, args: &[Value], env: &mut Environment) -> Result<Value, EvalError> {
    // 1. Arity check
    if args.len() != proc.params.len() {
        let proc_name = if proc.name.is_empty() { "anonymous procedure" } else { &proc.name };
        return Err(EvalError::Other(format!(
            "procedure '{}' expects {} argument(s), got {}",
            proc_name, proc.params.len(), args.len()
        )));
    }

    // 2. Memo lookup
    let memo_key = if proc.remember {
        let key = format!("{:?}", args);
        if let Some(cached) = proc.memo.borrow().get(&key).cloned() {
            return Ok(cached);
        }
        Some(key)
    } else {
        None
    };

    // 3. Save variables (params + locals)
    let mut saved: Vec<(String, Option<Value>)> = Vec::new();
    for name in proc.params.iter().chain(proc.locals.iter()) {
        let old = env.variables.remove(name);
        saved.push((name.clone(), old));
    }

    // 4. Bind parameters
    for (param_name, arg_value) in proc.params.iter().zip(args.iter()) {
        env.set_var(param_name, arg_value.clone());
    }
    // Locals are intentionally NOT initialized (accessing returns Symbol, Maple behavior)

    // 5. Execute body
    let result = match eval_stmt_sequence(&proc.body, env) {
        Ok(val) => Ok(val),
        Err(EvalError::EarlyReturn(val)) => Ok(val),
        Err(e) => Err(e),
    };

    // 6. Restore variables (always runs, regardless of success/error)
    for (name, old) in saved {
        match old {
            Some(v) => env.set_var(&name, v),
            None => { env.variables.remove(&name); }
        }
    }

    // 7. Memo store
    if let Some(key) = memo_key {
        if let Ok(ref val) = result {
            proc.memo.borrow_mut().insert(key, val.clone());
        }
    }

    // 8. Return result
    result
}

/// Extract an i64 from a Value (for loop bounds).
fn value_to_i64(val: &Value, context: &str) -> Result<i64, EvalError> {
    match val {
        Value::Integer(n) => n.0.to_i64().ok_or_else(|| {
            EvalError::Other(format!("{}: integer too large for loop bound", context))
        }),
        other => Err(EvalError::Other(format!(
            "{}: expected integer, got {}",
            context,
            other.type_name()
        ))),
    }
}

/// Evaluate a for loop.
///
/// Saves and restores the loop variable so it does not leak into the outer
/// scope. Returns the value of the last iteration's body, or Value::None
/// if zero iterations.
fn eval_for_loop(
    var: &str,
    from_node: &AstNode,
    to_node: &AstNode,
    by_node: Option<&AstNode>,
    body: &[Stmt],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    let from_val = eval_expr(from_node, env)?;
    let to_val = eval_expr(to_node, env)?;
    let start = value_to_i64(&from_val, "for-loop 'from'")?;
    let end = value_to_i64(&to_val, "for-loop 'to'")?;

    let step = match by_node {
        Some(node) => {
            let by_val = eval_expr(node, env)?;
            let s = value_to_i64(&by_val, "for-loop 'by'")?;
            if s == 0 {
                return Err(EvalError::Other("for-loop step cannot be zero".to_string()));
            }
            s
        }
        None => 1,
    };

    // Save the current value of the loop variable
    let saved = env.variables.remove(var);

    let mut i = start;

    // Run the loop body, ensuring we restore the variable on all exit paths
    let loop_result = (|| -> Result<Value, EvalError> {
        let mut last = Value::None;
        while (step > 0 && i <= end) || (step < 0 && i >= end) {
            env.set_var(var, Value::Integer(QInt::from(i)));
            last = eval_stmt_sequence(body, env)?;
            i += step;
        }
        Ok(last)
    })();

    // Restore the loop variable (on success, error, or EarlyReturn)
    match &saved {
        Some(old_val) => env.set_var(var, old_val.clone()),
        None => { env.variables.remove(var); }
    }

    loop_result
}

/// Evaluate add/mul/seq(expr, var=a..b) iteration functions.
///
/// These are special-cased to intercept the AST before evaluation:
/// - The first argument (body) is evaluated repeatedly per iteration.
/// - The second argument is `Compare(Eq, Variable(var), Range(lo, hi))`.
/// - The iteration variable is locally scoped (saved and restored).
fn eval_iteration_func(
    name: &str,
    args: &[AstNode],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: "2".to_string(),
            got: args.len(),
            signature: format!("{}(expr, var=a..b)", name),
        });
    }
    // Extract iteration variable and range from args[1]
    let (var_name, lo, hi) = match &args[1] {
        AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
            let var = match lhs.as_ref() {
                AstNode::Variable(v) => v.clone(),
                _ => return Err(EvalError::Other(
                    format!("{}: expected variable on left of =", name)
                )),
            };
            let (lo_node, hi_node) = match rhs.as_ref() {
                AstNode::Range { lo, hi } => (lo.as_ref(), hi.as_ref()),
                _ => return Err(EvalError::Other(
                    format!("{}: expected range (a..b) on right of =", name)
                )),
            };
            let lo_val = eval_expr(lo_node, env)?;
            let hi_val = eval_expr(hi_node, env)?;
            let lo_i = value_to_i64(&lo_val, &format!("{} range start", name))?;
            let hi_i = value_to_i64(&hi_val, &format!("{} range end", name))?;
            (var, lo_i, hi_i)
        }
        _ => return Err(EvalError::Other(
            format!("{}: second argument must be var=a..b", name)
        )),
    };

    // Save and restore iteration variable (same pattern as eval_for_loop)
    let saved = env.variables.remove(&var_name);

    let result = (|| -> Result<Value, EvalError> {
        match name {
            "add" => {
                let mut acc = Value::Integer(QInt::from(0i64));
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    acc = eval_add(acc, val, env)?;
                }
                Ok(acc)
            }
            "mul" => {
                let mut acc = Value::Integer(QInt::from(1i64));
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    acc = eval_mul(acc, val, env)?;
                }
                Ok(acc)
            }
            "seq" => {
                let mut items = Vec::new();
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    items.push(val);
                }
                Ok(Value::List(items))
            }
            _ => unreachable!(),
        }
    })();

    // Restore variable (even on error)
    match saved {
        Some(old_val) => env.set_var(&var_name, old_val),
        None => { env.variables.remove(&var_name); }
    }

    result
}

/// Evaluate a while-loop.
///
/// Repeatedly evaluates the condition and executes the body while the condition
/// is truthy. Returns the value of the last body execution, or Value::None if
/// the loop body never executes.
///
/// Includes a safety limit of 1,000,000 iterations to prevent infinite loops.
fn eval_while_loop(
    condition: &AstNode,
    body: &[Stmt],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    let mut last = Value::None;
    let max_iterations: u64 = 1_000_000;
    let mut count: u64 = 0;
    loop {
        let cond_val = eval_expr(condition, env)?;
        if !is_truthy(&cond_val)? {
            break;
        }
        last = eval_stmt_sequence(body, env)?;
        count += 1;
        if count >= max_iterations {
            return Err(EvalError::Other(
                "while loop exceeded maximum iteration count (1000000)".into(),
            ));
        }
    }
    Ok(last)
}

/// Evaluate an if/elif/else expression.
///
/// Returns the value of the selected branch, or Value::None if no branch
/// matches and there is no else clause.
fn eval_if_expr(
    condition: &AstNode,
    then_body: &[Stmt],
    elif_branches: &[(AstNode, Vec<Stmt>)],
    else_body: Option<&[Stmt]>,
    env: &mut Environment,
) -> Result<Value, EvalError> {
    // Check the main if condition
    let cond_val = eval_expr(condition, env)?;
    if is_truthy(&cond_val)? {
        return eval_stmt_sequence(then_body, env);
    }

    // Check elif branches
    for (elif_cond, elif_body) in elif_branches {
        let elif_val = eval_expr(elif_cond, env)?;
        if is_truthy(&elif_val)? {
            return eval_stmt_sequence(elif_body, env);
        }
    }

    // Else branch (if present)
    match else_body {
        Some(body) => eval_stmt_sequence(body, env),
        None => Ok(Value::None),
    }
}

// ---------------------------------------------------------------------------
// Unary negation
// ---------------------------------------------------------------------------

/// Negate a value.
fn eval_negate(val: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match val {
        Value::Series(fps) => Ok(Value::Series(arithmetic::negate(&fps))),
        Value::Integer(n) => Ok(Value::Integer(-n)),
        Value::Rational(r) => Ok(Value::Rational(-r)),
        Value::Symbol(name) => {
            let fps = symbol_to_series(&name, env);
            Ok(Value::Series(arithmetic::negate(&fps)))
        }
        Value::BivariateSeries(bs) => Ok(Value::BivariateSeries(bv::bivariate_negate(&bs))),
        Value::TrivariateSeries(ts) => Ok(Value::TrivariateSeries(tv::trivariate_negate(&ts))),
        Value::FractionalPowerSeries { inner, denom } => {
            Ok(Value::FractionalPowerSeries {
                inner: arithmetic::negate(&inner),
                denom,
            })
        }
        other => Err(EvalError::TypeError {
            operation: "unary -".to_string(),
            left: other.type_name().to_string(),
            right: String::new(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Binary arithmetic dispatch
// ---------------------------------------------------------------------------

/// Evaluate a binary operation on two values.
fn eval_binop(
    op: BinOp,
    left: Value,
    right: Value,
    env: &mut Environment,
) -> Result<Value, EvalError> {
    match op {
        BinOp::Add => eval_add(left, right, env),
        BinOp::Sub => eval_sub(left, right, env),
        BinOp::Mul => eval_mul(left, right, env),
        BinOp::Div => eval_div(left, right, env),
        BinOp::Pow => eval_pow(left, right, env),
    }
}

/// Convert a numeric value (Integer or Rational) to a constant FPS.
///
/// Uses the given `order` as truncation order. When promoting a scalar
/// to add/sub with an existing series, pass `fps.truncation_order()` to
/// preserve polynomial semantics (POLYNOMIAL_ORDER sentinel).
fn value_to_constant_fps(val: &Value, sym: qsym_core::symbol::SymbolId, order: i64) -> Option<FormalPowerSeries> {
    let qrat = match val {
        Value::Integer(n) => QRat::from(n.clone()),
        Value::Rational(r) => r.clone(),
        _ => return None,
    };
    Some(FormalPowerSeries::monomial(sym, qrat, 0, order))
}

/// Convert a numeric value to QRat for scalar operations.
fn value_to_qrat(val: &Value) -> Option<QRat> {
    match val {
        Value::Integer(n) => Some(QRat::from(n.clone())),
        Value::Rational(r) => Some(r.clone()),
        _ => None,
    }
}

/// Promote a symbol to a FPS monomial (var^1) with polynomial truncation order.
fn symbol_to_series(name: &str, env: &mut Environment) -> FormalPowerSeries {
    let sym_id = env.symbols.intern(name);
    FormalPowerSeries::monomial(sym_id, QRat::one(), 1, POLYNOMIAL_ORDER)
}

/// Try to promote a value to a FPS for mixed arithmetic with series.
/// Symbols become var^1 monomials; integers/rationals become constants.
fn value_to_series(val: &Value, env: &mut Environment) -> Option<FormalPowerSeries> {
    match val {
        Value::Symbol(name) => Some(symbol_to_series(name, env)),
        Value::Integer(n) => {
            let sym_q = env.sym_q;
            Some(FormalPowerSeries::monomial(sym_q, QRat::from(n.clone()), 0, POLYNOMIAL_ORDER))
        }
        Value::Rational(r) => {
            let sym_q = env.sym_q;
            Some(FormalPowerSeries::monomial(sym_q, r.clone(), 0, POLYNOMIAL_ORDER))
        }
        Value::Series(_) => {
            // Already a series; handled by dedicated match arms
            None
        }
        _ => None,
    }
}

fn eval_add(left: Value, right: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(a), Value::Series(b)) => Ok(Value::Series(arithmetic::add(a, b))),
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a.clone() + b.clone())),
        (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a.clone() + b.clone())),
        // Mixed numeric: promote to rational
        (Value::Integer(_), Value::Rational(_)) | (Value::Rational(_), Value::Integer(_)) => {
            let a = value_to_qrat(&left).unwrap();
            let b = value_to_qrat(&right).unwrap();
            Ok(Value::Rational(a + b))
        }
        // Series + scalar: promote scalar to constant FPS (match series truncation order)
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let const_fps = value_to_constant_fps(&right, fps.variable(), fps.truncation_order()).unwrap();
            Ok(Value::Series(arithmetic::add(fps, &const_fps)))
        }
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let const_fps = value_to_constant_fps(&left, fps.variable(), fps.truncation_order()).unwrap();
            Ok(Value::Series(arithmetic::add(&const_fps, fps)))
        }
        // Symbol + Series or Series + Symbol
        (Value::Symbol(_), Value::Series(fps)) => {
            let sym_fps = value_to_series(&left, env).unwrap();
            Ok(Value::Series(arithmetic::add(&sym_fps, fps)))
        }
        (Value::Series(fps), Value::Symbol(_)) => {
            let sym_fps = value_to_series(&right, env).unwrap();
            Ok(Value::Series(arithmetic::add(fps, &sym_fps)))
        }
        // Symbol involved: promote both sides to series
        (Value::Symbol(_), _) | (_, Value::Symbol(_)) => {
            let a = value_to_series(&left, env);
            let b = value_to_series(&right, env);
            if let (Some(fa), Some(fb)) = (a, b) {
                Ok(Value::Series(arithmetic::add(&fa, &fb)))
            } else {
                Err(EvalError::TypeError {
                    operation: "+".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                })
            }
        }
        // BivariateSeries + BivariateSeries
        (Value::BivariateSeries(a), Value::BivariateSeries(b)) => {
            Ok(Value::BivariateSeries(bv::bivariate_add(a, b)))
        }
        // BivariateSeries + scalar
        (Value::BivariateSeries(bs), _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let const_fps = FormalPowerSeries::monomial(bs.inner_variable, s, 0, bs.truncation_order);
            let rhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, const_fps);
            Ok(Value::BivariateSeries(bv::bivariate_add(bs, &rhs)))
        }
        (_, Value::BivariateSeries(bs)) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            let const_fps = FormalPowerSeries::monomial(bs.inner_variable, s, 0, bs.truncation_order);
            let lhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, const_fps);
            Ok(Value::BivariateSeries(bv::bivariate_add(&lhs, bs)))
        }
        // BivariateSeries + Series
        (Value::BivariateSeries(bs), Value::Series(fps)) => {
            let rhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, fps.clone());
            Ok(Value::BivariateSeries(bv::bivariate_add(bs, &rhs)))
        }
        (Value::Series(fps), Value::BivariateSeries(bs)) => {
            let lhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, fps.clone());
            Ok(Value::BivariateSeries(bv::bivariate_add(&lhs, bs)))
        }
        // FractionalPowerSeries + FractionalPowerSeries
        (Value::FractionalPowerSeries { inner: a, denom: da },
         Value::FractionalPowerSeries { inner: b, denom: db }) => {
            let (lcd, fa, fb) = unify_denoms(*da, *db);
            let ra = if fa == 1 { a.clone() } else { rescale_fps(a, fa) };
            let rb = if fb == 1 { b.clone() } else { rescale_fps(b, fb) };
            let result = arithmetic::add(&ra, &rb);
            Ok(simplify_fractional(result, lcd))
        }
        // FractionalPowerSeries + scalar
        (Value::FractionalPowerSeries { inner, denom }, _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let const_fps = FormalPowerSeries::monomial(inner.variable(), s, 0, inner.truncation_order());
            let result = arithmetic::add(inner, &const_fps);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        (_, Value::FractionalPowerSeries { inner, denom }) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            let const_fps = FormalPowerSeries::monomial(inner.variable(), s, 0, inner.truncation_order());
            let result = arithmetic::add(&const_fps, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        // FractionalPowerSeries + Series / Series + FractionalPowerSeries
        (Value::FractionalPowerSeries { inner, denom }, Value::Series(fps)) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::add(inner, &rescaled);
            Ok(simplify_fractional(result, *denom))
        }
        (Value::Series(fps), Value::FractionalPowerSeries { inner, denom }) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::add(&rescaled, inner);
            Ok(simplify_fractional(result, *denom))
        }
        // QProduct in add -> helpful error
        (Value::QProduct { .. }, _) | (_, Value::QProduct { .. }) => {
            Err(EvalError::Other(format!(
                "cannot add {} and {} -- qfactor result is a factorization, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // EtaQuotient in add -> helpful error
        (Value::EtaQuotient { .. }, _) | (_, Value::EtaQuotient { .. }) => {
            Err(EvalError::Other(format!(
                "cannot add {} and {} -- etamake result is an eta-quotient, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // JacobiProduct in add -> helpful error
        (Value::JacobiProduct(_), _) | (_, Value::JacobiProduct(_)) => {
            Err(EvalError::Other(format!(
                "cannot add {} and {} -- use jac2series() to expand first",
                left.type_name(), right.type_name()
            )))
        }
        _ => Err(EvalError::TypeError {
            operation: "+".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_sub(left: Value, right: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(a), Value::Series(b)) => Ok(Value::Series(arithmetic::sub(a, b))),
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a.clone() - b.clone())),
        (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a.clone() - b.clone())),
        // Mixed numeric
        (Value::Integer(_), Value::Rational(_)) | (Value::Rational(_), Value::Integer(_)) => {
            let a = value_to_qrat(&left).unwrap();
            let b = value_to_qrat(&right).unwrap();
            Ok(Value::Rational(a - b))
        }
        // Series - scalar (match series truncation order)
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let const_fps = value_to_constant_fps(&right, fps.variable(), fps.truncation_order()).unwrap();
            Ok(Value::Series(arithmetic::sub(fps, &const_fps)))
        }
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let const_fps = value_to_constant_fps(&left, fps.variable(), fps.truncation_order()).unwrap();
            Ok(Value::Series(arithmetic::sub(&const_fps, fps)))
        }
        // Symbol - Series or Series - Symbol
        (Value::Symbol(_), Value::Series(fps)) => {
            let sym_fps = value_to_series(&left, env).unwrap();
            Ok(Value::Series(arithmetic::sub(&sym_fps, fps)))
        }
        (Value::Series(fps), Value::Symbol(_)) => {
            let sym_fps = value_to_series(&right, env).unwrap();
            Ok(Value::Series(arithmetic::sub(fps, &sym_fps)))
        }
        // Symbol involved: promote both sides to series
        (Value::Symbol(_), _) | (_, Value::Symbol(_)) => {
            let a = value_to_series(&left, env);
            let b = value_to_series(&right, env);
            if let (Some(fa), Some(fb)) = (a, b) {
                Ok(Value::Series(arithmetic::sub(&fa, &fb)))
            } else {
                Err(EvalError::TypeError {
                    operation: "-".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                })
            }
        }
        // BivariateSeries - BivariateSeries
        (Value::BivariateSeries(a), Value::BivariateSeries(b)) => {
            Ok(Value::BivariateSeries(bv::bivariate_sub(a, b)))
        }
        // BivariateSeries - scalar
        (Value::BivariateSeries(bs), _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let const_fps = FormalPowerSeries::monomial(bs.inner_variable, s, 0, bs.truncation_order);
            let rhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, const_fps);
            Ok(Value::BivariateSeries(bv::bivariate_sub(bs, &rhs)))
        }
        (_, Value::BivariateSeries(bs)) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            let const_fps = FormalPowerSeries::monomial(bs.inner_variable, s, 0, bs.truncation_order);
            let lhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, const_fps);
            Ok(Value::BivariateSeries(bv::bivariate_sub(&lhs, bs)))
        }
        // BivariateSeries - Series
        (Value::BivariateSeries(bs), Value::Series(fps)) => {
            let rhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, fps.clone());
            Ok(Value::BivariateSeries(bv::bivariate_sub(bs, &rhs)))
        }
        (Value::Series(fps), Value::BivariateSeries(bs)) => {
            let lhs = BivariateSeries::from_single_term(bs.outer_variable.clone(), 0, fps.clone());
            Ok(Value::BivariateSeries(bv::bivariate_sub(&lhs, bs)))
        }
        // FractionalPowerSeries - FractionalPowerSeries
        (Value::FractionalPowerSeries { inner: a, denom: da },
         Value::FractionalPowerSeries { inner: b, denom: db }) => {
            let (lcd, fa, fb) = unify_denoms(*da, *db);
            let ra = if fa == 1 { a.clone() } else { rescale_fps(a, fa) };
            let rb = if fb == 1 { b.clone() } else { rescale_fps(b, fb) };
            let result = arithmetic::sub(&ra, &rb);
            Ok(simplify_fractional(result, lcd))
        }
        // FractionalPowerSeries - scalar
        (Value::FractionalPowerSeries { inner, denom }, _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let const_fps = FormalPowerSeries::monomial(inner.variable(), s, 0, inner.truncation_order());
            let result = arithmetic::sub(inner, &const_fps);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        (_, Value::FractionalPowerSeries { inner, denom }) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            let const_fps = FormalPowerSeries::monomial(inner.variable(), s, 0, inner.truncation_order());
            let result = arithmetic::sub(&const_fps, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        // FractionalPowerSeries - Series / Series - FractionalPowerSeries
        (Value::FractionalPowerSeries { inner, denom }, Value::Series(fps)) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::sub(inner, &rescaled);
            Ok(simplify_fractional(result, *denom))
        }
        (Value::Series(fps), Value::FractionalPowerSeries { inner, denom }) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::sub(&rescaled, inner);
            Ok(simplify_fractional(result, *denom))
        }
        // QProduct in sub -> helpful error
        (Value::QProduct { .. }, _) | (_, Value::QProduct { .. }) => {
            Err(EvalError::Other(format!(
                "cannot subtract {} and {} -- qfactor result is a factorization, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // EtaQuotient in sub -> helpful error
        (Value::EtaQuotient { .. }, _) | (_, Value::EtaQuotient { .. }) => {
            Err(EvalError::Other(format!(
                "cannot subtract {} and {} -- etamake result is an eta-quotient, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // JacobiProduct in sub -> helpful error
        (Value::JacobiProduct(_), _) | (_, Value::JacobiProduct(_)) => {
            Err(EvalError::Other(format!(
                "cannot subtract {} and {} -- use jac2series() to expand first",
                left.type_name(), right.type_name()
            )))
        }
        _ => Err(EvalError::TypeError {
            operation: "-".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_mul(left: Value, right: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(a), Value::Series(b)) => Ok(Value::Series(arithmetic::mul(a, b))),
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a.clone() * b.clone())),
        (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a.clone() * b.clone())),
        // Mixed numeric
        (Value::Integer(_), Value::Rational(_)) | (Value::Rational(_), Value::Integer(_)) => {
            let a = value_to_qrat(&left).unwrap();
            let b = value_to_qrat(&right).unwrap();
            Ok(Value::Rational(a * b))
        }
        // Scalar * Series
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            Ok(Value::Series(arithmetic::scalar_mul(&s, fps)))
        }
        // Series * Scalar
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            Ok(Value::Series(arithmetic::scalar_mul(&s, fps)))
        }
        // Symbol * Series or Series * Symbol
        (Value::Symbol(_), Value::Series(fps)) => {
            let sym_fps = value_to_series(&left, env).unwrap();
            Ok(Value::Series(arithmetic::mul(&sym_fps, fps)))
        }
        (Value::Series(fps), Value::Symbol(_)) => {
            let sym_fps = value_to_series(&right, env).unwrap();
            Ok(Value::Series(arithmetic::mul(fps, &sym_fps)))
        }
        // Symbol involved: promote both sides to series
        (Value::Symbol(_), _) | (_, Value::Symbol(_)) => {
            let a = value_to_series(&left, env);
            let b = value_to_series(&right, env);
            if let (Some(fa), Some(fb)) = (a, b) {
                Ok(Value::Series(arithmetic::mul(&fa, &fb)))
            } else {
                Err(EvalError::TypeError {
                    operation: "*".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                })
            }
        }
        // BivariateSeries * BivariateSeries
        (Value::BivariateSeries(a), Value::BivariateSeries(b)) => {
            Ok(Value::BivariateSeries(bv::bivariate_mul(a, b)))
        }
        // BivariateSeries * scalar
        (Value::BivariateSeries(bs), _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            Ok(Value::BivariateSeries(bv::bivariate_scalar_mul(&s, bs)))
        }
        (_, Value::BivariateSeries(bs)) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            Ok(Value::BivariateSeries(bv::bivariate_scalar_mul(&s, bs)))
        }
        // BivariateSeries * Series
        (Value::BivariateSeries(bs), Value::Series(fps)) => {
            Ok(Value::BivariateSeries(bv::bivariate_fps_mul(fps, bs)))
        }
        (Value::Series(fps), Value::BivariateSeries(bs)) => {
            Ok(Value::BivariateSeries(bv::bivariate_fps_mul(fps, bs)))
        }
        // FractionalPowerSeries * FractionalPowerSeries
        (Value::FractionalPowerSeries { inner: a, denom: da },
         Value::FractionalPowerSeries { inner: b, denom: db }) => {
            let (lcd, fa, fb) = unify_denoms(*da, *db);
            let ra = if fa == 1 { a.clone() } else { rescale_fps(a, fa) };
            let rb = if fb == 1 { b.clone() } else { rescale_fps(b, fb) };
            let result = arithmetic::mul(&ra, &rb);
            Ok(simplify_fractional(result, lcd))
        }
        // FractionalPowerSeries * scalar / scalar * FractionalPowerSeries
        (Value::FractionalPowerSeries { inner, denom }, _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let result = arithmetic::scalar_mul(&s, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        (_, Value::FractionalPowerSeries { inner, denom }) if value_to_qrat(&left).is_some() => {
            let s = value_to_qrat(&left).unwrap();
            let result = arithmetic::scalar_mul(&s, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        // FractionalPowerSeries * Series / Series * FractionalPowerSeries
        (Value::FractionalPowerSeries { inner, denom }, Value::Series(fps)) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::mul(inner, &rescaled);
            Ok(simplify_fractional(result, *denom))
        }
        (Value::Series(fps), Value::FractionalPowerSeries { inner, denom }) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = arithmetic::mul(&rescaled, inner);
            Ok(simplify_fractional(result, *denom))
        }
        // QProduct in mul -> helpful error
        (Value::QProduct { .. }, _) | (_, Value::QProduct { .. }) => {
            Err(EvalError::Other(format!(
                "cannot multiply {} and {} -- qfactor result is a factorization, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // EtaQuotient in mul -> helpful error
        (Value::EtaQuotient { .. }, _) | (_, Value::EtaQuotient { .. }) => {
            Err(EvalError::Other(format!(
                "cannot multiply {} and {} -- etamake result is an eta-quotient, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // JacobiProduct * JacobiProduct
        (Value::JacobiProduct(a), Value::JacobiProduct(b)) => {
            let mut combined = a.clone();
            combined.extend_from_slice(b);
            Ok(Value::JacobiProduct(normalize_jacobi_product(combined)))
        }
        _ => Err(EvalError::TypeError {
            operation: "*".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

/// Cap a series' truncation_order if it equals the POLYNOMIAL_ORDER sentinel.
/// Returns a truncated copy using `fallback` as the new order.
/// If the series is not POLYNOMIAL_ORDER, returns a clone unchanged.
fn cap_poly_order(fps: &FormalPowerSeries, fallback: i64) -> FormalPowerSeries {
    if fps.truncation_order() == POLYNOMIAL_ORDER {
        let mut coeffs = BTreeMap::new();
        for (&k, v) in fps.iter() {
            if k < fallback {
                coeffs.insert(k, v.clone());
            }
        }
        FormalPowerSeries::from_coeffs(fps.variable(), coeffs, fallback)
    } else {
        fps.clone()
    }
}

fn eval_div(left: Value, right: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(a), Value::Series(b)) => {
            let effective_order = match (a.truncation_order() == POLYNOMIAL_ORDER,
                                          b.truncation_order() == POLYNOMIAL_ORDER) {
                (true, true) => env.default_order,
                (true, false) => b.truncation_order(),
                (false, true) => a.truncation_order(),
                (false, false) => a.truncation_order().min(b.truncation_order()),
            };
            let a_work = cap_poly_order(a, effective_order);
            let b_work = cap_poly_order(b, effective_order);
            let inv = arithmetic::invert(&b_work);
            Ok(Value::Series(arithmetic::mul(&a_work, &inv)))
        }
        // Integer / Integer -> Rational
        (Value::Integer(a), Value::Integer(b)) => {
            let ra = QRat::from(a.clone());
            let rb = QRat::from(b.clone());
            Ok(Value::Rational(ra / rb))
        }
        (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a.clone() / b.clone())),
        // Mixed numeric
        (Value::Integer(_), Value::Rational(_)) | (Value::Rational(_), Value::Integer(_)) => {
            let a = value_to_qrat(&left).unwrap();
            let b = value_to_qrat(&right).unwrap();
            Ok(Value::Rational(a / b))
        }
        // Series / scalar -> scalar_mul by 1/scalar
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let inv_s = QRat::one() / s;
            Ok(Value::Series(arithmetic::scalar_mul(&inv_s, fps)))
        }
        // scalar / Series -> const_fps / series
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let effective_order = if fps.truncation_order() == POLYNOMIAL_ORDER {
                env.default_order
            } else {
                fps.truncation_order()
            };
            let const_fps = value_to_constant_fps(&left, fps.variable(), effective_order).unwrap();
            let fps_work = cap_poly_order(fps, effective_order);
            let inv = arithmetic::invert(&fps_work);
            Ok(Value::Series(arithmetic::mul(&const_fps, &inv)))
        }
        // Symbol / scalar -> series / scalar
        (Value::Symbol(_), _) if value_to_qrat(&right).is_some() => {
            let sym_fps = value_to_series(&left, env).unwrap();
            let s = value_to_qrat(&right).unwrap();
            let inv_s = QRat::one() / s;
            Ok(Value::Series(arithmetic::scalar_mul(&inv_s, &sym_fps)))
        }
        // Series / FractionalPowerSeries: primary use case (e.g. theta2(q,N)/q^(1/4))
        (Value::Series(fps), Value::FractionalPowerSeries { inner: div_fps, denom }) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = series_div_general(&rescaled, div_fps);
            Ok(simplify_fractional(result, *denom))
        }
        // FractionalPowerSeries / Series
        (Value::FractionalPowerSeries { inner, denom }, Value::Series(fps)) => {
            let rescaled = rescale_fps(fps, *denom);
            let result = series_div_general(inner, &rescaled);
            Ok(simplify_fractional(result, *denom))
        }
        // FractionalPowerSeries / FractionalPowerSeries
        (Value::FractionalPowerSeries { inner: a, denom: da },
         Value::FractionalPowerSeries { inner: b, denom: db }) => {
            let (lcd, fa, fb) = unify_denoms(*da, *db);
            let ra = if fa == 1 { a.clone() } else { rescale_fps(a, fa) };
            let rb = if fb == 1 { b.clone() } else { rescale_fps(b, fb) };
            let result = series_div_general(&ra, &rb);
            Ok(simplify_fractional(result, lcd))
        }
        // FractionalPowerSeries / scalar
        (Value::FractionalPowerSeries { inner, denom }, _) if value_to_qrat(&right).is_some() => {
            let s = value_to_qrat(&right).unwrap();
            let inv_s = QRat::one() / s;
            let result = arithmetic::scalar_mul(&inv_s, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        // scalar / FractionalPowerSeries
        (_, Value::FractionalPowerSeries { inner, denom }) if value_to_qrat(&left).is_some() => {
            let const_fps = FormalPowerSeries::monomial(inner.variable(), value_to_qrat(&left).unwrap(), 0, inner.truncation_order());
            let result = series_div_general(&const_fps, inner);
            Ok(Value::FractionalPowerSeries { inner: result, denom: *denom })
        }
        // QProduct in div -> helpful error
        (Value::QProduct { .. }, _) | (_, Value::QProduct { .. }) => {
            Err(EvalError::Other(format!(
                "cannot divide {} and {} -- qfactor result is a factorization, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // EtaQuotient in div -> helpful error
        (Value::EtaQuotient { .. }, _) | (_, Value::EtaQuotient { .. }) => {
            Err(EvalError::Other(format!(
                "cannot divide {} and {} -- etamake result is an eta-quotient, not a series",
                left.type_name(), right.type_name()
            )))
        }
        // JacobiProduct / JacobiProduct
        (Value::JacobiProduct(a), Value::JacobiProduct(b)) => {
            let mut combined = a.clone();
            for &(av, bv, exp) in b {
                combined.push((av, bv, -exp));
            }
            Ok(Value::JacobiProduct(normalize_jacobi_product(combined)))
        }
        _ => Err(EvalError::TypeError {
            operation: "/".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

/// Divide two FPS, handling the case where the divisor has a non-zero min_order.
/// For monomial divisors (single-term), uses efficient shift + scalar division.
/// For general divisors, shifts to normalize constant term, inverts, and multiplies.
fn series_div_general(numer: &FormalPowerSeries, denom_fps: &FormalPowerSeries) -> FormalPowerSeries {
    if let Some(min_ord) = denom_fps.min_order() {
        if denom_fps.num_nonzero() == 1 {
            // Pure monomial: c * q^k -- just shift and scale, no inversion needed
            let c = denom_fps.coeff(min_ord);
            let inv_c = QRat::one() / c;
            let shifted = arithmetic::shift(numer, -min_ord);
            return arithmetic::scalar_mul(&inv_c, &shifted);
        }
        if min_ord != 0 {
            // Shift divisor down so it has a constant term
            let shifted_denom = arithmetic::shift(denom_fps, -min_ord);
            let fallback = if numer.truncation_order() == POLYNOMIAL_ORDER { 20 } else { numer.truncation_order() };
            let capped = cap_poly_order(&shifted_denom, fallback);
            let inv = arithmetic::invert(&capped);
            let shifted_numer = arithmetic::shift(numer, -min_ord);
            return arithmetic::mul(&shifted_numer, &inv);
        }
    }
    // Divisor already has a constant term
    let fallback = if numer.truncation_order() == POLYNOMIAL_ORDER { 20 } else { numer.truncation_order() };
    let capped = cap_poly_order(denom_fps, fallback);
    let inv = arithmetic::invert(&capped);
    arithmetic::mul(numer, &inv)
}

/// Check if a FractionalPowerSeries can simplify back to a regular Series.
/// If all inner keys are multiples of `denom`, rescale keys down by `denom` and
/// return `Value::Series`; otherwise return `Value::FractionalPowerSeries`.
fn simplify_fractional(inner: FormalPowerSeries, denom: i64) -> Value {
    if denom == 1 || inner.iter().all(|(&k, _)| k % denom == 0) {
        let d = if denom == 1 { 1 } else { denom };
        let mut simplified = BTreeMap::new();
        for (&k, coeff) in inner.iter() {
            simplified.insert(k / d, coeff.clone());
        }
        let trunc = if inner.truncation_order() >= POLYNOMIAL_ORDER {
            POLYNOMIAL_ORDER
        } else {
            // Round up: ceil(trunc / denom)
            (inner.truncation_order() + d - 1) / d
        };
        Value::Series(FormalPowerSeries::from_coeffs(inner.variable(), simplified, trunc))
    } else {
        Value::FractionalPowerSeries { inner, denom }
    }
}

/// Rescale a FPS by multiplying all exponent keys by `factor`.
/// Used when lifting a regular series into fractional-exponent space.
fn rescale_fps(fps: &FormalPowerSeries, factor: i64) -> FormalPowerSeries {
    let mut rescaled = BTreeMap::new();
    for (&k, coeff) in fps.iter() {
        rescaled.insert(k * factor, coeff.clone());
    }
    let trunc = if fps.truncation_order() >= POLYNOMIAL_ORDER {
        POLYNOMIAL_ORDER
    } else {
        fps.truncation_order() * factor
    };
    FormalPowerSeries::from_coeffs(fps.variable(), rescaled, trunc)
}

/// Compute LCD of two denominators and return (lcd, factor_a, factor_b).
fn unify_denoms(d1: i64, d2: i64) -> (i64, i64, i64) {
    let g = gcd_i64(d1, d2);
    let lcd = d1 / g * d2;
    (lcd, lcd / d1, lcd / d2)
}

fn eval_pow(left: Value, right: Value, env: &mut Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(fps), Value::Integer(n)) => {
            let exp = n.0.to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let result = series_pow(fps, exp);
            Ok(Value::Series(result))
        }
        (Value::Integer(base), Value::Integer(exp)) => {
            let e = exp.0.to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            if e < 0 {
                // Integer ^ negative -> rational
                let rb = QRat::from(base.clone());
                let pos = (-e) as u64;
                let mut result = QRat::one();
                for _ in 0..pos {
                    result = result * rb.clone();
                }
                let inv = QRat::one() / result;
                Ok(Value::Rational(inv))
            } else {
                let result = base.pow_u32(e as u32);
                Ok(Value::Integer(result))
            }
        }
        (Value::Rational(base), Value::Integer(exp)) => {
            let e = exp.0.to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let abs_e = e.unsigned_abs() as u64;
            let mut result = QRat::one();
            for _ in 0..abs_e {
                result = result * base.clone();
            }
            if e < 0 {
                result = QRat::one() / result;
            }
            Ok(Value::Rational(result))
        }
        // Symbol ^ Integer -> monomial series
        (Value::Symbol(name), Value::Integer(n)) => {
            let exp = n.0.to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let sym_id = env.symbols.intern(name);
            let fps = FormalPowerSeries::monomial(sym_id, QRat::one(), exp, POLYNOMIAL_ORDER);
            Ok(Value::Series(fps))
        }
        // JacobiProduct ^ Integer
        (Value::JacobiProduct(factors), Value::Integer(n)) => {
            let exp = n.0.to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let scaled: Vec<_> = factors.iter().map(|&(a, b, e)| (a, b, e * exp)).collect();
            Ok(Value::JacobiProduct(normalize_jacobi_product(scaled)))
        }
        // Symbol ^ Rational (e.g. q^(n*n) where n*n produces Rational with denom=1,
        // or q^(1/4) for fractional powers)
        (Value::Symbol(name), Value::Rational(r)) => {
            let numer = r.0.numer().to_i64().ok_or_else(|| EvalError::Other(
                "exponent numerator too large".to_string(),
            ))?;
            let denom = r.0.denom().to_i64().ok_or_else(|| EvalError::Other(
                "exponent denominator too large".to_string(),
            ))?;
            if denom == 1 {
                // Integer exponent (existing behavior)
                let sym_id = env.symbols.intern(name);
                let fps = FormalPowerSeries::monomial(sym_id, QRat::one(), numer, POLYNOMIAL_ORDER);
                Ok(Value::Series(fps))
            } else {
                // Fractional exponent: q^(p/d)
                let sym_id = env.symbols.intern(name);
                let fps = FormalPowerSeries::monomial(sym_id, QRat::one(), numer, POLYNOMIAL_ORDER);
                Ok(Value::FractionalPowerSeries { inner: fps, denom })
            }
        }
        // Series ^ Rational (denom must be 1)
        (Value::Series(fps), Value::Rational(r)) => {
            let one = rug::Integer::from(1u32);
            if r.0.denom() != &one {
                return Err(EvalError::Other(format!(
                    "exponent must be an integer, got {}", r.0
                )));
            }
            let exp = r.0.numer().to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let result = series_pow(fps, exp);
            Ok(Value::Series(result))
        }
        // Integer ^ Rational (denom must be 1)
        (Value::Integer(base), Value::Rational(r)) => {
            let one = rug::Integer::from(1u32);
            if r.0.denom() != &one {
                return Err(EvalError::Other(format!(
                    "exponent must be an integer, got {}", r.0
                )));
            }
            let exp_int = QInt(r.0.numer().clone());
            eval_pow(Value::Integer(base.clone()), Value::Integer(exp_int), env)
        }
        // Rational ^ Rational (denom must be 1)
        (Value::Rational(base), Value::Rational(r)) => {
            let one = rug::Integer::from(1u32);
            if r.0.denom() != &one {
                return Err(EvalError::Other(format!(
                    "exponent must be an integer, got {}", r.0
                )));
            }
            let exp_int = QInt(r.0.numer().clone());
            eval_pow(Value::Rational(base.clone()), Value::Integer(exp_int), env)
        }
        // JacobiProduct ^ Rational (denom must be 1)
        (Value::JacobiProduct(factors), Value::Rational(r)) => {
            let one = rug::Integer::from(1u32);
            if r.0.denom() != &one {
                return Err(EvalError::Other(format!(
                    "exponent must be an integer, got {}", r.0
                )));
            }
            let exp = r.0.numer().to_i64().ok_or_else(|| EvalError::Other(
                "exponent too large".to_string(),
            ))?;
            let scaled: Vec<_> = factors.iter().map(|&(a, b, e)| (a, b, e * exp)).collect();
            Ok(Value::JacobiProduct(normalize_jacobi_product(scaled)))
        }
        _ => Err(EvalError::TypeError {
            operation: "^".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

/// Raise a FPS to an integer power.
fn series_pow(fps: &FormalPowerSeries, n: i64) -> FormalPowerSeries {
    if n == 0 {
        return FormalPowerSeries::one(fps.variable(), fps.truncation_order());
    }

    let base = if n < 0 {
        arithmetic::invert(fps)
    } else {
        fps.clone()
    };

    let abs_n = n.unsigned_abs();
    let mut result = base.clone();
    for _ in 1..abs_n {
        result = arithmetic::mul(&result, &base);
    }
    result
}

/// Normalize a JacobiProduct factor list: sort by (b, a), merge same (a, b)
/// by summing exponents, remove zero-exponent factors.
fn normalize_jacobi_product(mut factors: Vec<(i64, i64, i64)>) -> Vec<(i64, i64, i64)> {
    factors.sort_by_key(|&(a, b, _)| (b, a));
    let mut merged: Vec<(i64, i64, i64)> = Vec::new();
    for (a, b, exp) in factors {
        if let Some(last) = merged.last_mut() {
            if last.0 == a && last.1 == b {
                last.2 += exp;
                continue;
            }
        }
        merged.push((a, b, exp));
    }
    merged.retain(|&(_, _, exp)| exp != 0);
    merged
}

/// Compute r^n for integer n (positive, negative, or zero).
fn qrat_pow(r: &QRat, n: i64) -> QRat {
    if n == 0 {
        return QRat::one();
    }
    let (base, abs_n) = if n < 0 {
        (QRat::one() / r.clone(), n.unsigned_abs())
    } else {
        (r.clone(), n as u64)
    };
    let mut result = QRat::one();
    for _ in 0..abs_n {
        result = result * base.clone();
    }
    result
}

/// Expand a JacobiProduct to a FormalPowerSeries by computing each factor
/// via etaq(a, b, sym, order) and combining with mul/invert.
fn jacobi_product_to_fps(
    factors: &[(i64, i64, i64)],
    sym: SymbolId,
    order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(sym, order);
    for &(a, b, exp) in factors {
        let factor_fps = qseries::etaq(a, b, sym, order);
        if exp > 0 {
            for _ in 0..exp {
                result = arithmetic::mul(&result, &factor_fps);
            }
        } else if exp < 0 {
            let inv = arithmetic::invert(&factor_fps);
            for _ in 0..(-exp) {
                result = arithmetic::mul(&result, &inv);
            }
        }
        // exp == 0: skip (should not happen after normalization)
    }
    result
}

/// Expand a JacobiProduct to FPS using Garvan's JAC convention (for 2-arg jac2series).
///
/// In Garvan's convention:
/// - JAC(0, b) = (q^b; q^b)_inf  (NOT etaq(0,b) which gives zero)
/// - JAC(a, b) for 0 < a < b = (q^a;q^b)(q^{b-a};q^b)(q^b;q^b) = triple product
/// - JAC(a, b) for a < 0 or a >= b: reduce via ((a % b) + b) % b
fn jacobi_product_to_fps_garvan(
    factors: &[(i64, i64, i64)],
    sym: SymbolId,
    order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(sym, order);
    for &(a, b, exp) in factors {
        // Reduce a into [0, b) range
        let a_reduced = ((a % b) + b) % b;
        let factor_fps = if a_reduced == 0 {
            // JAC(0, b) = (q^b; q^b)_inf
            qseries::etaq(b, b, sym, order)
        } else {
            // JAC(a, b) for 0 < a < b = triple product (q^a;q^b)(q^{b-a};q^b)(q^b;q^b)
            qseries::jacprod(a_reduced, b, sym, order)
        };
        if exp > 0 {
            for _ in 0..exp {
                result = arithmetic::mul(&result, &factor_fps);
            }
        } else if exp < 0 {
            let inv = arithmetic::invert(&factor_fps);
            for _ in 0..(-exp) {
                result = arithmetic::mul(&result, &inv);
            }
        }
    }
    result
}

/// Format a JacobiProduct as explicit finite product notation.
/// E.g., "(1-q)(1-q^6)(1-q^11)..." for JAC(1,5) up to order.
fn format_product_notation(factors: &[(i64, i64, i64)], sym_name: &str, order: i64) -> String {
    if factors.is_empty() {
        return "1".to_string();
    }
    let mut numer_parts = Vec::new();
    let mut denom_parts = Vec::new();
    for &(a, b, exp) in factors {
        let abs_exp = exp.unsigned_abs() as i64;
        // Build the factor string: (1-q^a)(1-q^{a+b})(1-q^{a+2b})...
        let mut factor_strs = Vec::new();
        let mut k = a;
        while k > 0 && k < order {
            if k == 1 {
                factor_strs.push(format!("(1-{})", sym_name));
            } else {
                factor_strs.push(format!("(1-{}^{})", sym_name, k));
            }
            k += b;
        }
        let factor_block = factor_strs.join("");
        // Repeat for |exp| times
        for _ in 0..abs_exp {
            if exp > 0 {
                numer_parts.push(factor_block.clone());
            } else {
                denom_parts.push(factor_block.clone());
            }
        }
    }
    let numer = if numer_parts.is_empty() { "1".to_string() } else { numer_parts.join("") };
    if denom_parts.is_empty() {
        numer
    } else {
        format!("{}/{}", numer, denom_parts.join(""))
    }
}

// ---------------------------------------------------------------------------
// Function dispatch
// ---------------------------------------------------------------------------

/// Helper macro for dispatching mock theta functions (all take 1 arg: order).
macro_rules! dispatch_mock_theta {
    ($func:ident, $name:expr, $args:expr, $env:expr) => {{
        expect_args($name, $args, 1)?;
        let order = extract_i64($name, $args, 0)?;
        Ok(Value::Series(qseries::$func($env.sym_q, order)))
    }};
}

/// Euclidean GCD on i64 (handles negative inputs via abs).
fn gcd_i64(a: i64, b: i64) -> i64 {
    let (mut x, mut y) = (a.abs(), b.abs());
    while y != 0 {
        let tmp = y;
        y = x % y;
        x = tmp;
    }
    x
}

/// Shared helper for checkprod and findprod dispatch: check if series is a nice product.
///
/// Returns Value::List matching Garvan's output format:
/// - `[a, 1]` for nice product (all exponents < m_threshold)
/// - `[a, max_exp]` for not nice
/// - `[[a, c0], -1]` for non-integer leading coefficient
fn checkprod_impl(fps: &FormalPowerSeries, m_threshold: i64, q_order: i64) -> Value {
    // Step 1: Find valuation a
    let a = fps.min_order().unwrap_or(0);

    // Step 2: Get leading coefficient c0
    let c0 = fps.coeff(a);
    let one = rug::Integer::from(1u32);

    // Check integer-divisibility of leading coefficient
    if c0.denom() != &one {
        // Non-integer leading coefficient
        return Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(a)),
                Value::Rational(c0),
            ]),
            Value::Integer(QInt::from(-1i64)),
        ]);
    }

    // Step 3: Run prodmake (internally normalizes: strips q^a and divides by c0)
    let product = qseries::prodmake(fps, q_order);

    // Step 4: Find max |exponent|
    let max_exp = product.exponents.values()
        .map(|rat| {
            rat.numer().to_i64().unwrap_or(i64::MAX).abs()
        })
        .max()
        .unwrap_or(0);

    // Step 5: Return result
    if max_exp < m_threshold {
        Value::List(vec![
            Value::Integer(QInt::from(a)),
            Value::Integer(QInt::from(1i64)),
        ])
    } else {
        Value::List(vec![
            Value::Integer(QInt::from(a)),
            Value::Integer(QInt::from(max_exp)),
        ])
    }
}

/// Check if a checkprod result is "nice" (second element is Integer(1)).
/// Returns the valuation `a` if nice, None otherwise.
fn is_nice_checkprod_result(result: &Value) -> Option<i64> {
    if let Value::List(items) = result {
        if items.len() == 2 {
            if let (Value::Integer(a), Value::Integer(code)) = (&items[0], &items[1]) {
                if *code == QInt::from(1i64) {
                    return a.0.to_i64();
                }
            }
        }
    }
    None
}

/// Odometer-style increment of coefficient vector entries in [-max_coeff, max_coeff].
fn increment_coeffs(coeffs: &mut [i64], max_coeff: i64) -> bool {
    for c in coeffs.iter_mut().rev() {
        *c += 1;
        if *c <= max_coeff {
            return true;
        }
        *c = -max_coeff;
    }
    false
}

// ---------------------------------------------------------------------------
// Bivariate computation helpers
// ---------------------------------------------------------------------------

/// Compute tripleprod(z, q, T) with symbolic z via Jacobi triple product sum form.
///
/// Uses the Garvan convention:
///   (z;q)_inf * (q/z;q)_inf * (q;q)_inf = sum_{n=-inf}^{inf} (-1)^n * z^n * q^{n(n-1)/2}
///
/// Each term contributes (-1)^n at z^n with q-exponent n*(n-1)/2.
/// Include all n where n*(n-1)/2 < truncation_order and n*(n-1)/2 >= 0.
fn compute_tripleprod_bivariate(
    outer_var: &str,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // Bound: n*(n-1)/2 < T. For positive n: n < (1 + sqrt(1 + 8T))/2.
    // For negative n: |n|*(|n|+1)/2 < T (since n*(n-1)/2 = |n|*(|n|+1)/2 when n < 0).
    // Use a generous bound and filter.
    let bound = ((1.0 + (1.0 + 8.0 * truncation_order as f64).sqrt()) / 2.0).ceil() as i64 + 1;

    for n in -bound..=bound {
        let q_exp = n * (n - 1) / 2; // Garvan convention: n*(n-1)/2
        if q_exp < 0 || q_exp >= truncation_order {
            continue;
        }
        // Contribution: (-1)^n at z^n, q^{n*(n-1)/2}
        let sign = if n % 2 == 0 { QRat::one() } else { -QRat::one() };

        let entry = terms.entry(n).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old = entry.coeff(q_exp);
        entry.set_coeff(q_exp, old + sign);
    }

    // Remove zero entries
    terms.retain(|_, fps| !fps.is_zero());

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms,
        inner_variable: inner_var,
        truncation_order,
    }
}


/// Compute a bivariate q-Pochhammer infinite product `(c*z^{zp}*q^s; q)_inf`
/// where z is the symbolic outer variable, c is a concrete coefficient,
/// zp is the z-exponent per factor (+1 or -1), and s is the q-offset.
///
/// The product `prod_{k>=0}(1 - c * z^{zp} * q^{s+k})` is computed iteratively.
///
/// For negative offsets (s < 0), we use an internal q-shift: all FPS coefficients
/// are offset by `q_shift = |s| * z_bound`, so "q^{-1}" is stored at index
/// `q_shift - 1`. At the end, we shift each z^j coefficient back by `-q_shift`
/// and truncate to [0, T).
fn compute_pochhammer_bivariate(
    outer_var: &str,
    coeff: &QRat,
    z_power: i64,      // +1 or -1
    q_offset: i64,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    // Estimate max z-exponent range: each factor adds one z-exponent.
    // Number of factors is ~ truncation_order - q_offset.
    // But at each step the FPS gets more complex, so we bound z-range.
    // The product of N factors (1 - c*z*q^k) produces z^0..z^N.
    // We only need z-exponents whose q-contributions fit within [0, T).

    // For negative offsets, the minimum q-exponent that appears at z^j is roughly
    // j * q_offset (from the first |q_offset| factors contributing q^s with s < 0).
    // We need internal FPS to accommodate this.
    let neg_ext = if q_offset < 0 {
        // Each z^j gets shifted by j*q_offset at most. Max |j| is bounded by T.
        // But we don't need all -- be practical. Use |q_offset| * sqrt(T) headroom.
        let z_bound = ((2.0 * truncation_order as f64).sqrt().ceil() as i64).max(10);
        (-q_offset) * z_bound
    } else {
        0
    };
    let q_shift = neg_ext; // Amount we shift internal FPS upward
    let internal_trunc = truncation_order + q_shift;

    // Start with 1 at internal position q_shift (represents q^0 in true coordinates)
    let mut one_fps = FormalPowerSeries::zero(inner_var, internal_trunc);
    one_fps.set_coeff(q_shift, QRat::one()); // "1" in true coords = q^{q_shift} in internal
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
    terms.insert(0, one_fps);

    // Multiply by (1 - coeff * z^{zp} * q^{q_offset + k}) for k = 0, 1, 2, ...
    for k in 0.. {
        let exp = q_offset + k;
        if exp >= truncation_order {
            break;
        }
        // In internal coordinates, q^{exp} is stored at index q_shift + exp
        let internal_exp = q_shift + exp;
        if internal_exp < 0 {
            continue; // Below internal representable range
        }

        let mut new_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
        for (&z_j, f_j) in &terms {
            // Part 1: z^j contribution (identity)
            add_to_bv_terms(&mut new_terms, z_j, f_j);

            // Part 2: z^{j+zp} contribution = -coeff * q^{exp} * f_j
            // In internal coords: shift by internal_exp - q_shift = exp (in true coords)
            // But since f_j is in internal coords with shift q_shift,
            // we need to multiply f_j by q^{exp} = shift by exp in true coords
            // = shift by exp in internal coords (since internal = true + q_shift).
            // So the new internal FPS = shift(f_j, exp) * (-coeff)
            // where shift by exp in internal means: coefficient at internal p -> p + exp
            let shifted_z = z_j + z_power;
            let neg_coeff = -coeff.clone();
            let contrib = fps_shift_internal(f_j, exp, &neg_coeff, inner_var, internal_trunc);
            if !contrib.is_zero() {
                add_to_bv_terms(&mut new_terms, shifted_z, &contrib);
            }
        }
        terms = new_terms;
    }

    // Convert back: shift each z^j coefficient by -q_shift and truncate to [0, T)
    let mut final_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
    for (z_exp, fps) in terms {
        let mut truncated = FormalPowerSeries::zero(inner_var, truncation_order);
        for (&p, v) in fps.iter() {
            let true_p = p - q_shift; // Convert internal -> true
            if true_p >= 0 && true_p < truncation_order {
                truncated.set_coeff(true_p, v.clone());
            }
        }
        if !truncated.is_zero() {
            final_terms.insert(z_exp, truncated);
        }
    }

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms: final_terms,
        inner_variable: inner_var,
        truncation_order,
    }
}

/// Add an FPS to a bivariate terms map at z^{z_exp}, accumulating.
fn add_to_bv_terms(
    terms: &mut BTreeMap<i64, FormalPowerSeries>,
    z_exp: i64,
    fps: &FormalPowerSeries,
) {
    if let Some(existing) = terms.remove(&z_exp) {
        let sum = arithmetic::add(&existing, fps);
        if !sum.is_zero() { terms.insert(z_exp, sum); }
    } else {
        terms.insert(z_exp, fps.clone());
    }
}

/// Add an FPS to a trivariate terms map at (a_exp, b_exp), accumulating.
fn add_to_tv_terms(
    terms: &mut BTreeMap<(i64, i64), FormalPowerSeries>,
    key: (i64, i64),
    fps: &FormalPowerSeries,
) {
    if let Some(existing) = terms.remove(&key) {
        let sum = arithmetic::add(&existing, fps);
        if !sum.is_zero() { terms.insert(key, sum); }
    } else {
        terms.insert(key, fps.clone());
    }
}

/// Compute winquist(a, b, q, T) where BOTH `a` and `b` are symbolic outer variables.
///
/// Uses the 10-factor product decomposition (Garvan convention):
///   W(a,b,q) = (q;q)^2 * (a)(q/a)(b)(q/b)(ab)(q^2/(ab))(a/b)(qb/a)
/// where (x) denotes (x;q)_inf.
///
/// The 8 factors involving a and/or b are expanded as a trivariate product
/// (Laurent polynomial in a, b with FPS coefficients). The two (q;q)_inf
/// factors are multiplied into each FPS coefficient at the end.
///
/// Returns a TrivariateSeries: BTreeMap<(a_exp, b_exp), FPS in q>.
fn compute_winquist_two_symbolic(
    outer_var_a: &str,
    outer_var_b: &str,
    inner_var: SymbolId,
    truncation_order: i64,
) -> TrivariateSeries {
    use qsym_core::series::generator::euler_function_generator;

    // The 8 factor specs: (a_power, b_power, q_offset)
    // Factor 1: (a; q)          -> (1, 0, 0)
    // Factor 2: (q/a; q)        -> (-1, 0, 1)
    // Factor 3: (b; q)          -> (0, 1, 0)
    // Factor 4: (q/b; q)        -> (0, -1, 1)
    // Factor 5: (ab; q)         -> (1, 1, 0)
    // Factor 6: (q^2/(ab); q)   -> (-1, -1, 2)
    // Factor 7: (a/b; q)        -> (1, -1, 0)
    // Factor 8: (qb/a; q)       -> (-1, 1, 1)
    let tv_specs: [(i64, i64, i64); 8] = [
        ( 1,  0, 0),   // (a;q)
        (-1,  0, 1),   // (q/a;q)
        ( 0,  1, 0),   // (b;q)
        ( 0, -1, 1),   // (q/b;q)
        ( 1,  1, 0),   // (ab;q)
        (-1, -1, 2),   // (q^2/(ab);q)
        ( 1, -1, 0),   // (a/b;q)
        (-1,  1, 1),   // (qb/a;q)
    ];

    // All q_offsets are non-negative (min = 0), so no internal q-shift needed.
    let mut terms: BTreeMap<(i64, i64), FormalPowerSeries> = BTreeMap::new();
    {
        let one_fps = FormalPowerSeries::one(inner_var, truncation_order);
        terms.insert((0, 0), one_fps);
    }

    let neg_one = -QRat::one();

    // Process factor by factor: each is prod_{k>=0}(1 - a^{ap} * b^{bp} * q^{off+k})
    for &(ap, bp, q_offset) in &tv_specs {
        for k in 0.. {
            let true_exp = q_offset + k;
            if true_exp >= truncation_order { break; }

            let mut new_terms: BTreeMap<(i64, i64), FormalPowerSeries> = BTreeMap::new();
            for (&(ra, rb), f_j) in &terms {
                // Identity part
                add_to_tv_terms(&mut new_terms, (ra, rb), f_j);
                // Product part: -(1) * a^{ap} * b^{bp} * q^{true_exp} * f_j
                let shifted_key = (ra + ap, rb + bp);
                let contrib = fps_shift_internal(f_j, true_exp, &neg_one, inner_var, truncation_order);
                if !contrib.is_zero() {
                    add_to_tv_terms(&mut new_terms, shifted_key, &contrib);
                }
            }
            terms = new_terms;
        }
    }

    // Multiply every FPS coefficient by (q;q)^2
    let mut euler_gen = euler_function_generator(inner_var, truncation_order);
    euler_gen.ensure_order(truncation_order);
    let euler = euler_gen.into_series();
    let euler_sq = arithmetic::mul(&euler, &euler);

    let mut final_terms: BTreeMap<(i64, i64), FormalPowerSeries> = BTreeMap::new();
    for ((ra, rb), fps) in terms {
        let product = arithmetic::mul(&euler_sq, &fps);
        if !product.is_zero() {
            final_terms.insert((ra, rb), product);
        }
    }

    TrivariateSeries {
        outer_var_a: outer_var_a.to_string(),
        outer_var_b: outer_var_b.to_string(),
        terms: final_terms,
        inner_variable: inner_var,
        truncation_order,
    }
}

/// Internal FPS shift: shift FPS by `shift` positions (can be negative in internal coords)
/// and scale by `scale`. Only keeps terms in [0, trunc).
fn fps_shift_internal(
    fps: &FormalPowerSeries,
    shift: i64,
    scale: &QRat,
    inner_var: SymbolId,
    trunc: i64,
) -> FormalPowerSeries {
    if scale.is_zero() {
        return FormalPowerSeries::zero(inner_var, trunc);
    }
    let mut result = FormalPowerSeries::zero(inner_var, trunc);
    for (&p, v) in fps.iter() {
        let new_p = p + shift;
        if new_p >= 0 && new_p < trunc {
            result.set_coeff(new_p, scale.clone() * v.clone());
        }
    }
    result
}

/// Compute winquist(a, b, q, T) where `a` is the symbolic outer variable and
/// `b` is a concrete q-monomial.
///
/// Uses the 10-factor product decomposition (Garvan convention):
///   W(a,b,q) = (q;q)^2 * (a)(q/a)(b)(q/b)(ab)(q^2/(ab))(a/b)(qb/a)
/// where (x) denotes (x;q)_inf.
///
/// All 6 bivariate factors (involving a) are combined in a single loop with
/// a global internal q-shift to handle negative q-offsets. Concrete factors
/// ((b)(q/b)(q;q)^2) are multiplied in at the end.
fn compute_winquist_one_symbolic(
    outer_var: &str,
    b_mono: &QMonomial,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    use qsym_core::series::generator::{euler_function_generator, qpochhammer_inf_generator};

    let bc = &b_mono.coeff;
    let bp = b_mono.power;
    let inv_bc = QRat::one() / bc.clone();

    // Check for product-zero conditions
    let check_zero = |c: &QRat, offset: i64| -> bool {
        *c == QRat::one() && offset == 0
    };
    if check_zero(bc, bp) || check_zero(&inv_bc, 1 - bp) {
        return BivariateSeries::zero(outer_var.to_string(), inner_var, truncation_order);
    }

    // The 6 bivariate factors (c, z_power, q_offset):
    // Factor 1: (a; q)          = (1 * z^1 * q^0; q)
    // Factor 2: (q/a; q)        = (1 * z^{-1} * q^1; q)
    // Factor 5: (ab; q)         = (bc * z^1 * q^{bp}; q)
    // Factor 6: (q^2/(ab); q)   = (1/bc * z^{-1} * q^{2-bp}; q)
    // Factor 7: (a/b; q)        = (1/bc * z^1 * q^{-bp}; q)
    // Factor 8: (qb/a; q)       = (bc * z^{-1} * q^{1+bp}; q)
    let one = QRat::one();
    let bv_specs: Vec<(&QRat, i64, i64)> = vec![
        (&one,     1,  0),
        (&one,    -1,  1),
        (bc,       1,  bp),
        (&inv_bc, -1,  2 - bp),
        (&inv_bc,  1, -bp),
        (bc,      -1,  1 + bp),
    ];

    // Global q_shift: accommodate negative offsets across all factors.
    // At z^j, the minimum true q-exponent from the most negative offset factor
    // is j * min_offset (for z_power=+1 factors only). We need headroom for
    // the largest |z-exponent| that has significant contributions.
    let min_offset = bv_specs.iter().map(|&(_, _, off)| off).min().unwrap_or(0);
    let q_shift = if min_offset < 0 {
        // z-range grows as sqrt(2*T) per factor; with 6 factors, max|j| ~ 6*sqrt(2T).
        // But only z_power=+1 factors with negative offset contribute to negative q.
        // Use a generous but bounded estimate.
        let z_bound = ((2.0 * truncation_order as f64).sqrt().ceil() as i64 + 5).max(10);
        (-min_offset) * z_bound
    } else {
        0
    };
    let internal_trunc = truncation_order + q_shift;

    // Start with 1: z^0 with FPS "1" at internal index q_shift (= true q^0)
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
    {
        let mut one_fps = FormalPowerSeries::zero(inner_var, internal_trunc);
        one_fps.set_coeff(q_shift, QRat::one());
        terms.insert(0, one_fps);
    }

    // Process factor by factor: each is prod_{k>=0}(1 - c*z^{zp}*q^{off+k})
    for &(coeff, z_power, q_offset) in &bv_specs {
        for k in 0.. {
            let true_exp = q_offset + k;
            if true_exp >= truncation_order {
                break;
            }
            // Multiply current bivariate by (1 - coeff * z^{zp} * q^{true_exp})
            // In internal coords, shifting by true_exp means: index p -> p + true_exp
            let mut new_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
            for (&z_j, f_j) in &terms {
                // Identity part: z^j * f_j
                add_to_bv_terms(&mut new_terms, z_j, f_j);
                // Product part: -coeff * z^{j+zp} * q^{true_exp} * f_j
                let shifted_z = z_j + z_power;
                let neg_coeff = -coeff.clone();
                let contrib = fps_shift_internal(f_j, true_exp, &neg_coeff, inner_var, internal_trunc);
                if !contrib.is_zero() {
                    add_to_bv_terms(&mut new_terms, shifted_z, &contrib);
                }
            }
            terms = new_terms;
        }
    }

    // Concrete factors: (b;q)(q/b;q)(q;q)^2 -- computed at internal_trunc for headroom
    let mut gen3 = qpochhammer_inf_generator(bc.clone(), bp, inner_var, internal_trunc);
    gen3.ensure_order(internal_trunc);
    let f3 = gen3.into_series();

    let mut gen4 = qpochhammer_inf_generator(inv_bc.clone(), 1 - bp, inner_var, internal_trunc);
    gen4.ensure_order(internal_trunc);
    let f4 = gen4.into_series();

    let mut euler_gen = euler_function_generator(inner_var, internal_trunc);
    euler_gen.ensure_order(internal_trunc);
    let euler = euler_gen.into_series();
    let euler_sq = arithmetic::mul(&euler, &euler);
    let concrete = arithmetic::mul(&arithmetic::mul(&euler_sq, &f3), &f4);

    // Multiply bivariate (internal coords) by concrete (normal coords).
    // Internal bivariate index p represents true q^{p - q_shift}.
    // Concrete index j represents true q^j.
    // Convolution: product index p' = p + j represents true q^{(p-q_shift) + j} = q^{p'-q_shift}.
    // So the product is also in internal coords with the same q_shift.
    let mut multiplied_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
    for (&z_exp, coeff_fps) in &terms {
        let product = arithmetic::mul(&concrete, coeff_fps);
        if !product.is_zero() {
            multiplied_terms.insert(z_exp, product);
        }
    }

    // Convert internal -> true coordinates, keeping only [0, T)
    let mut final_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();
    for (z_exp, fps) in multiplied_terms {
        let mut truncated = FormalPowerSeries::zero(inner_var, truncation_order);
        for (&p, v) in fps.iter() {
            let true_p = p - q_shift;
            if true_p >= 0 && true_p < truncation_order {
                truncated.set_coeff(true_p, v.clone());
            }
        }
        if !truncated.is_zero() {
            final_terms.insert(z_exp, truncated);
        }
    }

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms: final_terms,
        inner_variable: inner_var,
        truncation_order,
    }
}

/// Format the quintuple product identity in product form.
fn format_quinprod_prodid(z: &str, q: &str) -> String {
    format!(
        "(-{z},{q})_inf * (-{q}/{z},{q})_inf * ({z}^2*{q},{q}^2)_inf * ({q}^2/{z}^2,{q}^2)_inf * ({q},{q})_inf",
        z = z, q = q
    )
}

/// Format the quintuple product identity in series form (product = series).
fn format_quinprod_seriesid(z: &str, q: &str) -> String {
    let prod_side = format_quinprod_prodid(z, q);
    format!(
        "{}\n  = sum(m=-inf..inf, ({z}^(3*m) - {z}^(-3*m-1)) * {q}^(m*(3*m+1)/2))",
        prod_side, z = z, q = q
    )
}

/// Compute quinprod(z, q, T) with symbolic z via quintuple product sum form.
///
/// quinprod(z, q, T) = sum_{m=-inf}^{inf} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
///
/// Each m contributes two terms:
///   +1 at z^{3m} with q-exponent m*(3m+1)/2
///   -1 at z^{-3m-1} with q-exponent m*(3m+1)/2
fn compute_quinprod_bivariate(
    outer_var: &str,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // Bound: m*(3m+1)/2 < T. Roughly |m| < sqrt(2T/3).
    let bound = ((1.0 + (1.0 + 24.0 * truncation_order as f64).sqrt()) / 6.0).ceil() as i64 + 1;

    for m in -bound..=bound {
        let q_exp = m * (3 * m + 1) / 2;
        if q_exp < 0 || q_exp >= truncation_order {
            continue;
        }

        // +1 at z^{3m}
        let z_exp_pos = 3 * m;
        let entry = terms.entry(z_exp_pos).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old = entry.coeff(q_exp);
        entry.set_coeff(q_exp, old + QRat::one());

        // -1 at z^{-3m-1}
        let z_exp_neg = -3 * m - 1;
        let entry2 = terms.entry(z_exp_neg).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old2 = entry2.coeff(q_exp);
        entry2.set_coeff(q_exp, old2 - QRat::one());
    }

    terms.retain(|_, fps| !fps.is_zero());

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms,
        inner_variable: inner_var,
        truncation_order,
    }
}

/// Dispatch a function call by name.
///
/// Resolves aliases, then matches against the canonical function name.
/// Groups 1-4 (25 functions) are implemented; Plan 03 fills in groups 5+.
pub fn dispatch(
    name: &str,
    args: &[Value],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    let canonical = resolve_alias(name);

    match canonical.as_str() {
        // =================================================================
        // Group 1: q-Pochhammer and Products (FUNC-01) -- 7 functions
        // =================================================================

        "aqprod" => {
            // Detect Maple-style: first arg is Series (monomial like q^2) or Symbol
            if !args.is_empty() && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
                // aqprod(a, q, n) or aqprod(a, q, n, order) or aqprod(a, q, infinity, order)
                let monomial = extract_monomial_from_arg(name, args, 0)?;
                let sym = extract_symbol_id(name, args, 1, env)?;

                if args.len() == 3 {
                    // aqprod(q^2, q, 5) -> finite product, n=args[2]
                    // Use POLYNOMIAL_ORDER as truncation so the result is an exact polynomial
                    let n = extract_i64(name, args, 2)?;
                    let result = qseries::aqprod(&monomial, sym, PochhammerOrder::Finite(n), POLYNOMIAL_ORDER);
                    Ok(Value::Series(result))
                } else if args.len() == 4 {
                    // aqprod(q^2, q, infinity, order) or aqprod(q^2, q, n, order)
                    let poch_order = match &args[2] {
                        Value::Infinity => PochhammerOrder::Infinite,
                        _ => {
                            let n = extract_i64(name, args, 2)?;
                            PochhammerOrder::Finite(n)
                        }
                    };
                    let order = extract_i64(name, args, 3)?;
                    let result = qseries::aqprod(&monomial, sym, poch_order, order);
                    Ok(Value::Series(result))
                } else {
                    Err(EvalError::WrongArgCount {
                        function: name.to_string(),
                        expected: "3 or 4 (Maple-style)".to_string(),
                        got: args.len(),
                        signature: "aqprod(monomial, q, n) or aqprod(monomial, q, n, order)".to_string(),
                    })
                }
            } else {
                // Legacy: aqprod(coeff_num, coeff_den, power, n_or_infinity, order)
                expect_args(name, args, 5)?;
                let cn = extract_i64(name, args, 0)?;
                let cd = extract_i64(name, args, 1)?;
                let power = extract_i64(name, args, 2)?;
                let poch_order = match &args[3] {
                    Value::Infinity => PochhammerOrder::Infinite,
                    _ => {
                        let n = extract_i64(name, args, 3)?;
                        PochhammerOrder::Finite(n)
                    }
                };
                let order = extract_i64(name, args, 4)?;
                let monomial = QMonomial::new(QRat::from((cn, cd)), power);
                let result = qseries::aqprod(&monomial, env.sym_q, poch_order, order);
                Ok(Value::Series(result))
            }
        }

        "qbin" => {
            if args.len() == 3 && matches!(&args[0], Value::Symbol(_)) {
                // Garvan: qbin(q, m, n) -- exact polynomial
                let sym = extract_symbol_id(name, args, 0, env)?;
                let m = extract_i64(name, args, 1)?;
                let n = extract_i64(name, args, 2)?;
                // Exact polynomial of degree m*(n-m), use tight truncation
                // then re-wrap with POLYNOMIAL_ORDER sentinel for display
                let degree = if m >= 0 && m <= n { m * (n - m) } else { 0 };
                let tight_order = degree + 1;
                let computed = qseries::qbin(n, m, sym, tight_order);
                // Re-wrap with POLYNOMIAL_ORDER sentinel so display omits O(...)
                let coeffs: BTreeMap<i64, QRat> = computed.iter()
                    .map(|(&k, v)| (k, v.clone()))
                    .collect();
                let result = FormalPowerSeries::from_coeffs(sym, coeffs, POLYNOMIAL_ORDER);
                Ok(Value::Series(result))
            } else if args.len() == 4 && matches!(&args[2], Value::Symbol(_)) {
                // Extended: qbin(n, k, q, T) -- with explicit variable and truncation
                let n = extract_i64(name, args, 0)?;
                let k = extract_i64(name, args, 1)?;
                let sym = extract_symbol_id(name, args, 2, env)?;
                let order = extract_i64(name, args, 3)?;
                let result = qseries::qbin(n, k, sym, order);
                Ok(Value::Series(result))
            } else {
                // Legacy: qbin(n, k, order)
                expect_args(name, args, 3)?;
                let n = extract_i64(name, args, 0)?;
                let k = extract_i64(name, args, 1)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::qbin(n, k, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        "etaq" => {
            if args.len() >= 2 && matches!(&args[0], Value::Symbol(_)) {
                let sym = extract_symbol_id(name, args, 0, env)?;
                if args.len() == 3 && matches!(&args[1], Value::List(_)) {
                    // Multi-delta: etaq(q, [d1, d2, ...], T)
                    let deltas = extract_i64_list(name, args, 1)?;
                    let order = extract_i64(name, args, 2)?;
                    if deltas.is_empty() {
                        return Err(EvalError::Other(
                            format!("{}: delta list must not be empty", name),
                        ));
                    }
                    let mut result = FormalPowerSeries::one(sym, order);
                    for d in &deltas {
                        if *d <= 0 {
                            return Err(EvalError::Other(
                                format!("{}: each delta must be positive, got {}", name, d),
                            ));
                        }
                        let factor = qseries::etaq(*d, *d, sym, order);
                        result = arithmetic::mul(&result, &factor);
                    }
                    Ok(Value::Series(result))
                } else {
                    // Single delta: etaq(q, b, T)
                    expect_args(name, args, 3)?;
                    let b = extract_i64(name, args, 1)?;
                    let order = extract_i64(name, args, 2)?;
                    let result = qseries::etaq(b, b, sym, order);
                    Ok(Value::Series(result))
                }
            } else {
                // Legacy: etaq(b, t, order)
                expect_args(name, args, 3)?;
                let b = extract_i64(name, args, 0)?;
                let t = extract_i64(name, args, 1)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::etaq(b, t, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        "jacprod" => {
            if args.len() == 4 && matches!(&args[2], Value::Symbol(_)) {
                // Maple: jacprod(a, b, q, T) = JAC(a,b) / JAC(b,3b)
                let a_val = extract_i64(name, args, 0)?;
                let b_val = extract_i64(name, args, 1)?;
                let sym = extract_symbol_id(name, args, 2, env)?;
                let order = extract_i64(name, args, 3)?;
                let jac_ab = qseries::jacprod(a_val, b_val, sym, order);
                let jac_b3b = qseries::jacprod(b_val, 3 * b_val, sym, order);
                let inv_b3b = arithmetic::invert(&jac_b3b);
                let result = arithmetic::mul(&jac_ab, &inv_b3b);
                Ok(Value::Series(result))
            } else {
                // Legacy: jacprod(a, b, order)
                expect_args(name, args, 3)?;
                let a = extract_i64(name, args, 0)?;
                let b = extract_i64(name, args, 1)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::jacprod(a, b, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        "tripleprod" => {
            if args.len() == 3 && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
                // Maple: tripleprod(z, q, T)
                // Check if first arg is a Symbol with a DIFFERENT name from the q-variable
                let is_symbolic_outer = match (&args[0], &args[1]) {
                    (Value::Symbol(z_name), Value::Symbol(q_name)) => z_name != q_name,
                    _ => false,
                };

                if is_symbolic_outer {
                    // Bivariate path: symbolic z
                    let outer_name = match &args[0] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let sym = extract_symbol_id(name, args, 1, env)?;
                    let order = extract_i64(name, args, 2)?;
                    let result = compute_tripleprod_bivariate(&outer_name, sym, order);
                    Ok(Value::BivariateSeries(result))
                } else {
                    // Existing monomial path (z is q-monomial or Symbol("q") treated as q^1)
                    let monomial = extract_monomial_from_arg(name, args, 0)?;
                    let sym = extract_symbol_id(name, args, 1, env)?;
                    let order = extract_i64(name, args, 2)?;
                    let result = qseries::tripleprod(&monomial, sym, order);
                    Ok(Value::Series(result))
                }
            } else {
                // Legacy: tripleprod(coeff_num, coeff_den, power, order)
                expect_args(name, args, 4)?;
                let cn = extract_i64(name, args, 0)?;
                let cd = extract_i64(name, args, 1)?;
                let power = extract_i64(name, args, 2)?;
                let order = extract_i64(name, args, 3)?;
                let monomial = QMonomial::new(QRat::from((cn, cd)), power);
                let result = qseries::tripleprod(&monomial, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        "quinprod" => {
            if args.len() == 3 && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
                // Identity display modes: quinprod(z, q, prodid) or quinprod(z, q, seriesid)
                if let Value::Symbol(mode) = &args[2] {
                    let mode_str = mode.as_str();
                    if mode_str == "prodid" || mode_str == "seriesid" {
                        let z_str = match &args[0] {
                            Value::Symbol(s) => s.clone(),
                            _ => "z".to_string(),
                        };
                        let q_str = match &args[1] {
                            Value::Symbol(s) => s.clone(),
                            _ => "q".to_string(),
                        };
                        let identity = if mode_str == "prodid" {
                            format_quinprod_prodid(&z_str, &q_str)
                        } else {
                            format_quinprod_seriesid(&z_str, &q_str)
                        };
                        println!("{}", identity);
                        return Ok(Value::String(identity));
                    }
                }

                // Maple: quinprod(z, q, T)
                // Check if first arg is a Symbol with a DIFFERENT name from the q-variable
                let is_symbolic_outer = match (&args[0], &args[1]) {
                    (Value::Symbol(z_name), Value::Symbol(q_name)) => z_name != q_name,
                    _ => false,
                };

                if is_symbolic_outer {
                    // Bivariate path: symbolic z
                    let outer_name = match &args[0] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let sym = extract_symbol_id(name, args, 1, env)?;
                    let order = extract_i64(name, args, 2)?;
                    let result = compute_quinprod_bivariate(&outer_name, sym, order);
                    Ok(Value::BivariateSeries(result))
                } else {
                    // Existing monomial path (z is q-monomial or Symbol("q") treated as q^1)
                    let monomial = extract_monomial_from_arg(name, args, 0)?;
                    let sym = extract_symbol_id(name, args, 1, env)?;
                    let order = extract_i64(name, args, 2)?;
                    let result = qseries::quinprod(&monomial, sym, order);
                    Ok(Value::Series(result))
                }
            } else {
                // Legacy: quinprod(coeff_num, coeff_den, power, order)
                expect_args(name, args, 4)?;
                let cn = extract_i64(name, args, 0)?;
                let cd = extract_i64(name, args, 1)?;
                let power = extract_i64(name, args, 2)?;
                let order = extract_i64(name, args, 3)?;
                let monomial = QMonomial::new(QRat::from((cn, cd)), power);
                let result = qseries::quinprod(&monomial, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        "winquist" => {
            if args.len() == 4 && matches!(&args[2], Value::Symbol(_)) {
                // Maple: winquist(a, b, q, T)
                // Check which args are symbolic (different from q variable)
                let a_is_symbolic = match (&args[0], &args[2]) {
                    (Value::Symbol(a_name), Value::Symbol(q_name)) => a_name != q_name,
                    _ => false,
                };
                let b_is_symbolic = match (&args[1], &args[2]) {
                    (Value::Symbol(b_name), Value::Symbol(q_name)) => b_name != q_name,
                    _ => false,
                };

                if a_is_symbolic && b_is_symbolic {
                    let a_name = match &args[0] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let b_name = match &args[1] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let sym = extract_symbol_id(name, args, 2, env)?;
                    let order = extract_i64(name, args, 3)?;
                    let result = compute_winquist_two_symbolic(&a_name, &b_name, sym, order);
                    Ok(Value::TrivariateSeries(result))
                } else if a_is_symbolic {
                    // a is symbolic, b is concrete
                    let outer_name = match &args[0] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let b_mono = extract_monomial_from_arg(name, args, 1)?;
                    let sym = extract_symbol_id(name, args, 2, env)?;
                    let order = extract_i64(name, args, 3)?;
                    let result = compute_winquist_one_symbolic(&outer_name, &b_mono, sym, order);
                    Ok(Value::BivariateSeries(result))
                } else if b_is_symbolic {
                    // b is symbolic, a is concrete -- swap roles
                    // winquist(a, b) factors symmetrically: TP(a)*TP(b)*TP(ab)*TP(a/b)
                    // Swapping a<->b: TP(b)*TP(a)*TP(ba)*TP(b/a) -- same up to TP(a/b) vs TP(b/a)
                    // TP(x) = TP(1/x) * x (Jacobi identity), and in product they cancel
                    let outer_name = match &args[1] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
                    let a_mono = extract_monomial_from_arg(name, args, 0)?;
                    let sym = extract_symbol_id(name, args, 2, env)?;
                    let order = extract_i64(name, args, 3)?;
                    let result = compute_winquist_one_symbolic(&outer_name, &a_mono, sym, order);
                    Ok(Value::BivariateSeries(result))
                } else {
                    // Both are concrete monomials -- existing path
                    let a = extract_monomial_from_arg(name, args, 0)?;
                    let b = extract_monomial_from_arg(name, args, 1)?;
                    let sym = extract_symbol_id(name, args, 2, env)?;
                    let order = extract_i64(name, args, 3)?;
                    let result = qseries::winquist(&a, &b, sym, order);
                    Ok(Value::Series(result))
                }
            } else {
                // Legacy: winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)
                expect_args(name, args, 7)?;
                let a_cn = extract_i64(name, args, 0)?;
                let a_cd = extract_i64(name, args, 1)?;
                let a_p = extract_i64(name, args, 2)?;
                let b_cn = extract_i64(name, args, 3)?;
                let b_cd = extract_i64(name, args, 4)?;
                let b_p = extract_i64(name, args, 5)?;
                let order = extract_i64(name, args, 6)?;
                let a = QMonomial::new(QRat::from((a_cn, a_cd)), a_p);
                let b = QMonomial::new(QRat::from((b_cn, b_cd)), b_p);
                let result = qseries::winquist(&a, &b, env.sym_q, order);
                Ok(Value::Series(result))
            }
        }

        // =================================================================
        // Group 2: Partitions (FUNC-02) -- 7 functions
        // =================================================================

        "numbpart" => {
            if args.len() == 1 {
                // numbpart(n) -- count all partitions of n
                let n = extract_i64(name, args, 0)?;
                let result = qseries::partition_count(n);
                Ok(Value::Integer(QInt(result.0.numer().clone())))
            } else if args.len() == 2 {
                // numbpart(n, m) -- count partitions of n with max part <= m
                let n = extract_i64(name, args, 0)?;
                let m = extract_i64(name, args, 1)?;
                if m <= 0 {
                    // 0 parts means only p(0,m)=1 when n=0
                    if n == 0 {
                        Ok(Value::Integer(QInt::from(1i64)))
                    } else {
                        Ok(Value::Integer(QInt::from(0i64)))
                    }
                } else {
                    // Use bounded_parts_gf(m, sym, n+1) and extract coefficient of q^n
                    let gf = qseries::bounded_parts_gf(m, env.sym_q, n + 1);
                    let coeff = gf.coeff(n);
                    Ok(Value::Integer(QInt(coeff.0.numer().clone())))
                }
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1 or 2".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "partition_gf" => {
            // partition_gf(order)
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::partition_gf(env.sym_q, order);
            Ok(Value::Series(result))
        }

        "distinct_parts_gf" => {
            // distinct_parts_gf(order)
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::distinct_parts_gf(env.sym_q, order);
            Ok(Value::Series(result))
        }

        "odd_parts_gf" => {
            // odd_parts_gf(order)
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::odd_parts_gf(env.sym_q, order);
            Ok(Value::Series(result))
        }

        "bounded_parts_gf" => {
            // bounded_parts_gf(max_part, order)
            expect_args(name, args, 2)?;
            let max_part = extract_i64(name, args, 0)?;
            let order = extract_i64(name, args, 1)?;
            let result = qseries::bounded_parts_gf(max_part, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "rank_gf" => {
            // rank_gf(z_num, z_den, order)
            expect_args(name, args, 3)?;
            let z_num = extract_i64(name, args, 0)?;
            let z_den = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let z = QRat::from((z_num, z_den));
            let result = qseries::rank_gf(&z, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "crank_gf" => {
            // crank_gf(z_num, z_den, order)
            expect_args(name, args, 3)?;
            let z_num = extract_i64(name, args, 0)?;
            let z_den = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let z = QRat::from((z_num, z_den));
            let result = qseries::crank_gf(&z, env.sym_q, order);
            Ok(Value::Series(result))
        }

        // =================================================================
        // Group 3: Theta Functions (FUNC-03) -- 3 functions
        // =================================================================

        "theta2" => {
            if args.len() == 1 {
                // theta2(T) -- legacy 1-arg form
                let order = extract_i64(name, args, 0)?;
                let result = qseries::theta2(env.sym_q, order);
                Ok(Value::Series(result))
            } else if args.len() == 2 {
                // theta2(q, T) -- Garvan 2-arg form
                // Monomial args not supported for theta2 (half-integer exponents)
                if let Value::Series(_) = &args[0] {
                    return Err(EvalError::Other(
                        "theta2: monomial argument q^k not supported, use subs(q=q^k, theta2(q,T)) instead".into()
                    ));
                }
                let sym = extract_symbol_id(name, args, 0, env)?;
                let order = extract_i64(name, args, 1)?;
                let result = qseries::theta2(sym, order);
                Ok(Value::Series(result))
            } else if args.len() == 3 {
                // theta2(a, q, T) -- Garvan 3-arg form
                // When a == q (same variable), reduces to standard theta2
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::theta2(sym, order);
                Ok(Value::Series(result))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1, 2, or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "theta3" => {
            if args.len() == 1 {
                // theta3(T) -- legacy 1-arg form
                let order = extract_i64(name, args, 0)?;
                let result = qseries::theta3(env.sym_q, order);
                Ok(Value::Series(result))
            } else if args.len() == 2 {
                // theta3(q, T) or theta3(q^k, T) -- Garvan 2-arg form with monomial support
                match &args[0] {
                    Value::Series(mono) => {
                        // theta3(q^k, T) -- compute theta3 then scale exponents by k
                        let terms: Vec<_> = mono.iter().collect();
                        if terms.len() == 1 {
                            let (&exp, coeff) = terms[0];
                            if *coeff == QRat::one() && exp > 0 {
                                let order = extract_i64(name, args, 1)?;
                                let sym = mono.variable();
                                // Compute theta3(q, order) then scale all exponents by exp
                                let base = qseries::theta3(sym, order);
                                let mut new_coeffs = std::collections::BTreeMap::new();
                                for (&e, c) in base.iter() {
                                    new_coeffs.insert(e * exp, c.clone());
                                }
                                let result = FormalPowerSeries::from_coeffs(sym, new_coeffs, order * exp);
                                return Ok(Value::Series(result));
                            }
                        }
                        Err(EvalError::Other(format!(
                            "{}: first argument must be a variable or q^k monomial", name
                        )))
                    }
                    _ => {
                        let sym = extract_symbol_id(name, args, 0, env)?;
                        let order = extract_i64(name, args, 1)?;
                        let result = qseries::theta3(sym, order);
                        Ok(Value::Series(result))
                    }
                }
            } else if args.len() == 3 {
                // theta3(a, q, T) -- Garvan 3-arg form
                // When a == q (same variable), reduces to standard theta3
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::theta3(sym, order);
                Ok(Value::Series(result))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1, 2, or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "theta4" => {
            if args.len() == 1 {
                // theta4(T) -- legacy 1-arg form
                let order = extract_i64(name, args, 0)?;
                let result = qseries::theta4(env.sym_q, order);
                Ok(Value::Series(result))
            } else if args.len() == 2 {
                // theta4(q, T) or theta4(q^k, T) -- Garvan 2-arg form with monomial support
                match &args[0] {
                    Value::Series(mono) => {
                        // theta4(q^k, T) -- compute theta4 then scale exponents by k
                        let terms: Vec<_> = mono.iter().collect();
                        if terms.len() == 1 {
                            let (&exp, coeff) = terms[0];
                            if *coeff == QRat::one() && exp > 0 {
                                let order = extract_i64(name, args, 1)?;
                                let sym = mono.variable();
                                // Compute theta4(q, order) then scale all exponents by exp
                                let base = qseries::theta4(sym, order);
                                let mut new_coeffs = std::collections::BTreeMap::new();
                                for (&e, c) in base.iter() {
                                    new_coeffs.insert(e * exp, c.clone());
                                }
                                let result = FormalPowerSeries::from_coeffs(sym, new_coeffs, order * exp);
                                return Ok(Value::Series(result));
                            }
                        }
                        Err(EvalError::Other(format!(
                            "{}: first argument must be a variable or q^k monomial", name
                        )))
                    }
                    _ => {
                        let sym = extract_symbol_id(name, args, 0, env)?;
                        let order = extract_i64(name, args, 1)?;
                        let result = qseries::theta4(sym, order);
                        Ok(Value::Series(result))
                    }
                }
            } else if args.len() == 3 {
                // theta4(a, q, T) -- Garvan 3-arg form
                // When a == q (same variable), reduces to standard theta4
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::theta4(sym, order);
                Ok(Value::Series(result))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1, 2, or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        // =================================================================
        // Group 4: Series Analysis (FUNC-04) -- 9 functions
        // =================================================================

        "sift" => {
            // Maple: sift(s, q, n, k, T)
            expect_args(name, args, 5)?;
            let fps = extract_series(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let n = extract_i64(name, args, 2)?;
            let k = extract_i64(name, args, 3)?;
            let t = extract_i64(name, args, 4)?;
            if n <= 0 {
                return Err(EvalError::Other(format!(
                    "sift: Argument 3 (n): modulus must be positive, got {}", n
                )));
            }
            if k < 0 || k >= n {
                return Err(EvalError::Other(format!(
                    "sift: Argument 4 (k): residue must satisfy 0 <= k < n={}, got {}", n, k
                )));
            }
            // Cap T at the series truncation order, then truncate input before sifting
            let effective_t = t.min(fps.truncation_order());
            let truncated_input = if effective_t < fps.truncation_order() {
                let mut coeffs = std::collections::BTreeMap::new();
                for (&exp, c) in fps.iter() {
                    if exp < effective_t {
                        coeffs.insert(exp, c.clone());
                    }
                }
                FormalPowerSeries::from_coeffs(fps.variable(), coeffs, effective_t)
            } else {
                fps
            };
            let result = qseries::sift(&truncated_input, n, k);
            Ok(Value::Series(result))
        }

        "qdegree" => {
            // qdegree(series)
            expect_args(name, args, 1)?;
            let fps = extract_series(name, args, 0)?;
            match qseries::qdegree(&fps) {
                Some(d) => Ok(Value::Integer(QInt::from(d))),
                None => Ok(Value::None),
            }
        }

        "lqdegree" => {
            // lqdegree(series)
            expect_args(name, args, 1)?;
            let fps = extract_series(name, args, 0)?;
            match qseries::lqdegree(&fps) {
                Some(d) => Ok(Value::Integer(QInt::from(d))),
                None => Ok(Value::None),
            }
        }

        "lqdegree0" => {
            // Garvan: lqdegree0(qexp) -- lowest q-degree (alias of lqdegree for FPS)
            expect_args(name, args, 1)?;
            let fps = extract_series(name, args, 0)?;
            match fps.min_order() {
                Some(d) => Ok(Value::Integer(QInt::from(d))),
                None => Ok(Value::None),
            }
        }

        "prodmake" => {
            // Maple: prodmake(f, q, T)
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let max_n = extract_i64(name, args, 2)?;
            let result = qseries::prodmake(&fps, max_n);
            Ok(infinite_product_form_to_value(&result))
        }

        "etamake" => {
            // Maple: etamake(f, q, T)
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let max_n = extract_i64(name, args, 2)?;
            let result = qseries::etamake(&fps, max_n);
            Ok(eta_quotient_to_value(&result))
        }

        "jacprodmake" => {
            // Maple: jacprodmake(f, q, T) or jacprodmake(f, q, T, P)
            if args.len() == 3 {
                let fps = extract_series(name, args, 0)?;
                let _sym = extract_symbol_id(name, args, 1, env)?;
                let max_n = extract_i64(name, args, 2)?;
                let result = qseries::jacprodmake(&fps, max_n);
                Ok(jacobi_product_form_to_value(&result))
            } else if args.len() == 4 {
                let fps = extract_series(name, args, 0)?;
                let _sym = extract_symbol_id(name, args, 1, env)?;
                let max_n = extract_i64(name, args, 2)?;
                let pp = extract_i64(name, args, 3)?;
                if pp <= 0 {
                    return Err(EvalError::Other(format!(
                        "jacprodmake: Argument 4 (P): period filter must be positive, got {}", pp
                    )));
                }
                let result = qseries::jacprodmake_with_period_filter(&fps, max_n, pp);
                Ok(jacobi_product_form_to_value(&result))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "3 or 4".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "mprodmake" => {
            // Maple: mprodmake(f, q, T)
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let max_n = extract_i64(name, args, 2)?;
            let result = qseries::mprodmake(&fps, max_n);
            Ok(btreemap_i64_to_value(&result))
        }

        "qetamake" => {
            // Maple: qetamake(f, q, T)
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let max_n = extract_i64(name, args, 2)?;
            let result = qseries::qetamake(&fps, max_n);
            Ok(q_eta_form_to_value(&result))
        }

        "qfactor" => {
            // Maple: qfactor(f, q) or qfactor(f, T) or qfactor(f, q, T)
            if args.len() == 2 {
                let fps = extract_series(name, args, 0)?;
                match &args[1] {
                    Value::Symbol(_) => {
                        // qfactor(f, q) -- existing form, explicit variable
                        let _sym = extract_symbol_id(name, args, 1, env)?;
                    }
                    Value::Integer(_) => {
                        // qfactor(f, T) -- Garvan 2-arg form, implicit variable q
                        let _t = extract_i64(name, args, 1)?;
                    }
                    other => {
                        return Err(EvalError::ArgType {
                            function: name.to_string(),
                            arg_index: 1,
                            expected: "Symbol or Integer",
                            got: other.type_name().to_string(),
                        });
                    }
                }
                let result = qseries::qfactor(&fps);
                Ok(q_factorization_to_value(&result))
            } else if args.len() == 3 {
                let fps = extract_series(name, args, 0)?;
                let _sym = extract_symbol_id(name, args, 1, env)?;
                let _t = extract_i64(name, args, 2)?;
                // T parameter accepted for Maple compat but our qfactor is already degree-bounded
                let result = qseries::qfactor(&fps);
                Ok(q_factorization_to_value(&result))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "2 or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "zqfactor" => {
            // Maple: zqfactor(f, z, q) or zqfactor(f, z, q, maxdeg)
            expect_args_range(name, args, 3, 4)?;
            let bseries = match &args[0] {
                Value::BivariateSeries(bs) => bs.clone(),
                other => {
                    return Err(EvalError::ArgType {
                        function: name.to_string(),
                        arg_index: 0,
                        expected: "bivariate series",
                        got: other.type_name().to_string(),
                    });
                }
            };
            let _z_sym = extract_symbol_id(name, args, 1, env)?;
            let _q_sym = extract_symbol_id(name, args, 2, env)?;
            let result = qseries::zqfactor(&bseries);
            let s = format_zqfactor_result(&result, &bseries.outer_variable);
            println!("{}", s);
            Ok(Value::String(s))
        }

        "checkmult" => {
            // Garvan: checkmult(QS, T) or checkmult(QS, T, 'yes')
            expect_args_range(name, args, 2, 3)?;
            let fps = extract_series(name, args, 0)?;
            let t = extract_i64(name, args, 1)?;
            let print_all = args.len() == 3 && matches!(&args[2], Value::String(s) if s == "yes");

            let mut failures: Vec<(i64, i64)> = Vec::new();
            let half_t = t / 2;
            'outer: for m in 2..=half_t {
                for n in m..=half_t {
                    if m * n > t { break; }
                    if gcd_i64(m, n) != 1 { continue; }
                    let fm = fps.coeff(m);
                    let fn_ = fps.coeff(n);
                    let fmn = fps.coeff(m * n);
                    if fm.clone() * fn_ != fmn {
                        failures.push((m, n));
                        if !print_all { break 'outer; }
                    }
                }
            }

            if failures.is_empty() {
                println!("MULTIPLICATIVE");
                Ok(Value::Integer(QInt::from(1i64)))
            } else {
                for (m, n) in &failures {
                    println!("NOT MULTIPLICATIVE at ({}, {})", m, n);
                }
                Ok(Value::Integer(QInt::from(0i64)))
            }
        }

        "checkprod" => {
            // Garvan: checkprod(f, M, Q) -- check if series is nice product
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let m_threshold = extract_i64(name, args, 1)?;
            let q_order = extract_i64(name, args, 2)?;
            Ok(checkprod_impl(&fps, m_threshold, q_order))
        }

        // =================================================================
        // Group 5: Relation Discovery (FUNC-05) -- 15 functions
        // =================================================================

        // Pattern D: target + list of candidates

        "findlincombo" => {
            // Maple: findlincombo(f, L, SL, q, topshift)
            expect_args(name, args, 5)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let labels = extract_symbol_list(name, args, 2)?;
            let _sym = extract_symbol_id(name, args, 3, env)?;
            let topshift = extract_i64(name, args, 4)?;
            if labels.len() != candidates.len() {
                return Err(EvalError::Other(format!(
                    "{}: SL has {} labels but L has {} series",
                    name, labels.len(), candidates.len()
                )));
            }
            validate_unique_labels(name, &labels)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findlincombo(&target, &refs, topshift) {
                Some(coeffs) => {
                    let s = format_linear_combo(&coeffs, &labels);
                    println!("{}", s);
                    Ok(Value::String(s))
                }
                None => {
                    println!("NOT A LINEAR COMBO.");
                    Ok(Value::None)
                }
            }
        }

        "findhomcombo" => {
            // Maple: findhomcombo(f, L, q, n, topshift) -- NO SL
            expect_args(name, args, 5)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let _sym = extract_symbol_id(name, args, 2, env)?;
            let degree = extract_i64(name, args, 3)?;
            let topshift = extract_i64(name, args, 4)?;
            let labels = default_labels(candidates.len());
            let monomials = qseries::generate_monomials(candidates.len(), degree);
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findhomcombo(&target, &refs, degree, topshift) {
                Some(coeffs) => {
                    let s = format_polynomial_expr(&coeffs, &monomials, &labels);
                    println!("{}", s);
                    Ok(Value::String(s))
                }
                None => {
                    println!("NOT A HOMOGENEOUS COMBO.");
                    Ok(Value::None)
                }
            }
        }

        "findnonhomcombo" => {
            // Maple: findnonhomcombo(f, L, q, n, topshift) -- NO SL
            expect_args(name, args, 5)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let _sym = extract_symbol_id(name, args, 2, env)?;
            let degree = extract_i64(name, args, 3)?;
            let topshift = extract_i64(name, args, 4)?;
            let labels = default_labels(candidates.len());
            let monomials = qseries::generate_nonhom_monomials(candidates.len(), degree);
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findnonhomcombo(&target, &refs, degree, topshift) {
                Some(coeffs) => {
                    let s = format_polynomial_expr(&coeffs, &monomials, &labels);
                    println!("{}", s);
                    Ok(Value::String(s))
                }
                None => {
                    println!("NOT A NON-HOMOGENEOUS COMBO.");
                    Ok(Value::None)
                }
            }
        }

        "findlincombomodp" => {
            // Maple: findlincombomodp(f, L, SL, p, q, topshift) -- p BEFORE q
            expect_args(name, args, 6)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let labels = extract_symbol_list(name, args, 2)?;
            let p = extract_i64(name, args, 3)?;
            let _sym = extract_symbol_id(name, args, 4, env)?;
            let topshift = extract_i64(name, args, 5)?;
            if !is_prime(p) {
                return Err(EvalError::Other(format!(
                    "{}: {} is not prime", name, p
                )));
            }
            if labels.len() != candidates.len() {
                return Err(EvalError::Other(format!(
                    "{}: SL has {} labels but L has {} series",
                    name, labels.len(), candidates.len()
                )));
            }
            validate_unique_labels(name, &labels)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findlincombomodp(&target, &refs, p, topshift) {
                Some(coeffs) => {
                    let s = format_linear_combo_modp(&coeffs, &labels, p);
                    println!("{}", s);
                    Ok(Value::String(s))
                }
                None => {
                    println!("NOT A LINEAR COMBO MOD {}.", p);
                    Ok(Value::None)
                }
            }
        }

        "findhomcombomodp" => {
            // Maple: findhomcombomodp(f, L, p, q, n, topshift) -- NO SL, p before q
            expect_args(name, args, 6)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let p = extract_i64(name, args, 2)?;
            let _sym = extract_symbol_id(name, args, 3, env)?;
            let degree = extract_i64(name, args, 4)?;
            let topshift = extract_i64(name, args, 5)?;
            if !is_prime(p) {
                return Err(EvalError::Other(format!(
                    "{}: {} is not prime", name, p
                )));
            }
            let labels = default_labels(candidates.len());
            let monomials = qseries::generate_monomials(candidates.len(), degree);
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findhomcombomodp(&target, &refs, p, degree, topshift) {
                Some(coeffs) => {
                    let s = format_polynomial_expr_modp(&coeffs, &monomials, &labels, p);
                    println!("{}", s);
                    Ok(Value::String(s))
                }
                None => {
                    println!("NOT A HOMOGENEOUS COMBO MOD {}.", p);
                    Ok(Value::None)
                }
            }
        }

        // Pattern E: list of series

        "findhom" => {
            // Maple: findhom(L, q, n, topshift)
            expect_args(name, args, 4)?;
            let series_list = extract_series_list(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let degree = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let labels = default_labels(series_list.len());
            let monomials = qseries::generate_monomials(series_list.len(), degree);
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findhom(&refs, degree, topshift);
            if rows.is_empty() {
                println!("NO HOMOGENEOUS RELATIONS FOUND.");
                return Ok(Value::List(vec![]));
            }
            let mut exprs = Vec::new();
            for row in &rows {
                let s = format_polynomial_expr(row, &monomials, &labels);
                println!("{}", s);
                exprs.push(Value::String(s));
            }
            Ok(Value::List(exprs))
        }

        "findnonhom" => {
            // Maple: findnonhom(L, q, n, topshift)
            expect_args(name, args, 4)?;
            let series_list = extract_series_list(name, args, 0)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let degree = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let labels = default_labels(series_list.len());
            let monomials = qseries::generate_nonhom_monomials(series_list.len(), degree);
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findnonhom(&refs, degree, topshift);
            if rows.is_empty() {
                println!("NO NON-HOMOGENEOUS RELATIONS FOUND.");
                return Ok(Value::List(vec![]));
            }
            let mut exprs = Vec::new();
            for row in &rows {
                let s = format_polynomial_expr(row, &monomials, &labels);
                println!("{}", s);
                exprs.push(Value::String(s));
            }
            Ok(Value::List(exprs))
        }

        "findhommodp" => {
            // Maple: findhommodp(L, p, q, n, topshift) -- p BEFORE q
            expect_args(name, args, 5)?;
            let series_list = extract_series_list(name, args, 0)?;
            let p = extract_i64(name, args, 1)?;
            let _sym = extract_symbol_id(name, args, 2, env)?;
            let degree = extract_i64(name, args, 3)?;
            let topshift = extract_i64(name, args, 4)?;
            if !is_prime(p) {
                return Err(EvalError::Other(format!(
                    "{}: {} is not prime", name, p
                )));
            }
            let labels = default_labels(series_list.len());
            let monomials = qseries::generate_monomials(series_list.len(), degree);
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findhommodp(&refs, p, degree, topshift);
            if rows.is_empty() {
                println!("NO HOMOGENEOUS RELATIONS MOD {} FOUND.", p);
                return Ok(Value::List(vec![]));
            }
            let mut exprs = Vec::new();
            for row in &rows {
                let s = format_polynomial_expr_modp(row, &monomials, &labels, p);
                println!("{}", s);
                exprs.push(Value::String(s));
            }
            Ok(Value::List(exprs))
        }

        "findmaxind" => {
            // Garvan: findmaxind(L, T) -- 2 args, no q
            expect_args(name, args, 2)?;
            let series_list = extract_series_list(name, args, 0)?;
            let topshift = extract_i64(name, args, 1)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let indices = qseries::findmaxind(&refs, topshift);
            // Return 1-based indices matching Garvan convention
            let nxfl: Vec<Value> = indices.iter()
                .map(|&i| Value::Integer(QInt::from((i + 1) as i64)))
                .collect();
            let display: Vec<i64> = indices.iter().map(|&i| (i + 1) as i64).collect();
            println!("{:?}", display);
            Ok(Value::List(nxfl))
        }

        "findprod" => {
            // Garvan: findprod(FL, T, M, Q) -- exhaustive search for product identities
            expect_args(name, args, 4)?;
            let series_list = extract_series_list(name, args, 0)?;
            let max_coeff = extract_i64(name, args, 1)?;
            let m_threshold = extract_i64(name, args, 2)?;
            let q_order = extract_i64(name, args, 3)?;

            let k = series_list.len();
            let mut results: Vec<Value> = Vec::new();

            // Iterate coefficient vectors from [-max_coeff, ..., -max_coeff]
            // to [max_coeff, ..., max_coeff] using odometer increment
            let mut coeffs = vec![-max_coeff; k];
            loop {
                // Skip zero vector
                if coeffs.iter().any(|&c| c != 0) {
                    // Primitive vector check: gcd of absolute values == 1
                    let g = coeffs.iter().fold(0i64, |acc, &c| gcd_i64(acc, c.abs()));
                    if g <= 1 {
                        // Form linear combination
                        let trunc = q_order.min(
                            series_list.iter().map(|s| s.truncation_order()).min().unwrap()
                        );
                        let var = series_list[0].variable();
                        let mut combo = FormalPowerSeries::zero(var, trunc);
                        for (s, &c) in series_list.iter().zip(coeffs.iter()) {
                            if c == 0 { continue; }
                            let scaled = arithmetic::scalar_mul(&QRat::from((c, 1i64)), s);
                            combo = arithmetic::add(&combo, &scaled);
                        }

                        if !combo.is_zero() {
                            let result = checkprod_impl(&combo, m_threshold, q_order);
                            if let Some(a) = is_nice_checkprod_result(&result) {
                                let mut row = vec![Value::Integer(QInt::from(a))];
                                row.extend(coeffs.iter().map(|&c| Value::Integer(QInt::from(c))));
                                results.push(Value::List(row));
                            }
                        }
                    }
                }
                if !increment_coeffs(&mut coeffs, max_coeff) {
                    break;
                }
            }
            Ok(Value::List(results))
        }

        "findcong" => {
            // Maple: findcong(QS, T, [LM], [XSET]) -- 2 to 4 args
            expect_args_range(name, args, 2, 4)?;
            let fps = extract_series(name, args, 0)?;
            let t = extract_i64(name, args, 1)?;
            let lm = if args.len() >= 3 {
                Some(extract_i64(name, args, 2)?)
            } else {
                None
            };
            let xset: HashSet<i64> = if args.len() >= 4 {
                extract_i64_list(name, args, 3)?.into_iter().collect()
            } else {
                HashSet::new()
            };
            let results = qseries::findcong_garvan(&fps, t, lm, &xset);
            if results.is_empty() {
                println!("NO CONGRUENCES FOUND.");
            }
            for c in &results {
                println!("[{}, {}, {}]", c.residue_b, c.modulus_m, c.divisor_r);
            }
            Ok(Value::List(
                results.iter().map(|c| Value::List(vec![
                    Value::Integer(QInt::from(c.residue_b)),
                    Value::Integer(QInt::from(c.modulus_m)),
                    Value::Integer(QInt::from(c.divisor_r)),
                ])).collect(),
            ))
        }

        // Pattern F: two series

        "findpoly" => {
            // Maple: findpoly(x, y, q, dx, dy, [check]) -- 5 or 6 args
            expect_args_range(name, args, 5, 6)?;
            let x = extract_series(name, args, 0)?;
            let y = extract_series(name, args, 1)?;
            let _sym = extract_symbol_id(name, args, 2, env)?;
            let deg_x = extract_i64(name, args, 3)?;
            let deg_y = extract_i64(name, args, 4)?;
            let check = if args.len() == 6 {
                Some(extract_i64(name, args, 5)?)
            } else {
                None
            };
            // Fixed topshift=10 matching Garvan's dim2 := dim1 + 10
            match qseries::findpoly(&x, &y, deg_x, deg_y, 10) {
                Some(rel) => {
                    let s = format_findpoly_result(&rel);
                    println!("The polynomial is");
                    println!("{}", s);
                    if let Some(check_order) = check {
                        let verified = verify_findpoly_result(&rel, &x, &y, check_order);
                        if verified {
                            println!("The relation has been verified to O(q^{})", check_order);
                        } else {
                            println!("WARNING: verification FAILED at O(q^{})", check_order);
                        }
                    }
                    Ok(Value::String(s))
                }
                None => {
                    println!("NO polynomial relation found.");
                    Ok(Value::None)
                }
            }
        }

        // =================================================================
        // Group 6: Hypergeometric (FUNC-06) -- 9 functions
        // =================================================================

        "phi" => {
            // phi(upper_list, lower_list, z_num, z_den, z_pow, order)
            let (series, order) = build_hypergeometric(name, args)?;
            let result = qseries::eval_phi(&series, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "psi" => {
            // psi(upper_list, lower_list, z_num, z_den, z_pow, order)
            let (series, order) = build_bilateral(name, args)?;
            let result = qseries::eval_psi(&series, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "try_summation" => {
            // try_summation(upper_list, lower_list, z_num, z_den, z_pow, order)
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::try_all_summations(&series, env.sym_q, order) {
                qseries::SummationResult::ClosedForm(fps) => Ok(Value::Series(fps)),
                qseries::SummationResult::NotApplicable => Ok(Value::None),
            }
        }

        "heine1" => {
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::heine_transform_1(&series, env.sym_q, order) {
                Some(tr) => Ok(Value::Pair(
                    Box::new(Value::Series(tr.prefactor)),
                    Box::new(Value::Series(qseries::eval_phi(&tr.transformed, env.sym_q, order))),
                )),
                None => Ok(Value::None),
            }
        }

        "heine2" => {
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::heine_transform_2(&series, env.sym_q, order) {
                Some(tr) => Ok(Value::Pair(
                    Box::new(Value::Series(tr.prefactor)),
                    Box::new(Value::Series(qseries::eval_phi(&tr.transformed, env.sym_q, order))),
                )),
                None => Ok(Value::None),
            }
        }

        "heine3" => {
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::heine_transform_3(&series, env.sym_q, order) {
                Some(tr) => Ok(Value::Pair(
                    Box::new(Value::Series(tr.prefactor)),
                    Box::new(Value::Series(qseries::eval_phi(&tr.transformed, env.sym_q, order))),
                )),
                None => Ok(Value::None),
            }
        }

        "sears_transform" => {
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::sears_transform(&series, env.sym_q, order) {
                Some(tr) => Ok(Value::Pair(
                    Box::new(Value::Series(tr.prefactor)),
                    Box::new(Value::Series(qseries::eval_phi(&tr.transformed, env.sym_q, order))),
                )),
                None => Ok(Value::None),
            }
        }

        "watson_transform" => {
            let (series, order) = build_hypergeometric(name, args)?;
            match qseries::watson_transform(&series, env.sym_q, order) {
                Some(tr) => Ok(Value::Pair(
                    Box::new(Value::Series(tr.prefactor)),
                    Box::new(Value::Series(qseries::eval_phi(&tr.transformed, env.sym_q, order))),
                )),
                None => Ok(Value::None),
            }
        }

        "find_transformation_chain" => {
            // find_transformation_chain(src_upper, src_lower, src_z_n, src_z_d, src_z_p,
            //                           tgt_upper, tgt_lower, tgt_z_n, tgt_z_d, tgt_z_p,
            //                           max_depth, order)
            expect_args(name, args, 12)?;
            let src_upper = extract_monomial_list(name, args, 0)?;
            let src_lower = extract_monomial_list(name, args, 1)?;
            let src_zn = extract_i64(name, args, 2)?;
            let src_zd = extract_i64(name, args, 3)?;
            let src_zp = extract_i64(name, args, 4)?;
            let tgt_upper = extract_monomial_list(name, args, 5)?;
            let tgt_lower = extract_monomial_list(name, args, 6)?;
            let tgt_zn = extract_i64(name, args, 7)?;
            let tgt_zd = extract_i64(name, args, 8)?;
            let tgt_zp = extract_i64(name, args, 9)?;
            let max_depth = extract_i64(name, args, 10)? as usize;
            let order = extract_i64(name, args, 11)?;
            let source = HypergeometricSeries {
                upper: src_upper,
                lower: src_lower,
                argument: QMonomial::new(QRat::from((src_zn, src_zd)), src_zp),
            };
            let target = HypergeometricSeries {
                upper: tgt_upper,
                lower: tgt_lower,
                argument: QMonomial::new(QRat::from((tgt_zn, tgt_zd)), tgt_zp),
            };
            let result = qseries::find_transformation_chain(&source, &target, max_depth, env.sym_q, order);
            Ok(transformation_chain_result_to_value(&result))
        }

        // =================================================================
        // Group 7: Mock Theta / Appell-Lerch / Bailey (FUNC-07) -- 27 functions
        // =================================================================

        // 20 mock theta functions (all take 1 arg: order)
        "mock_theta_f3" => dispatch_mock_theta!(mock_theta_f3, name, args, env),
        "mock_theta_phi3" => dispatch_mock_theta!(mock_theta_phi3, name, args, env),
        "mock_theta_psi3" => dispatch_mock_theta!(mock_theta_psi3, name, args, env),
        "mock_theta_chi3" => dispatch_mock_theta!(mock_theta_chi3, name, args, env),
        "mock_theta_omega3" => dispatch_mock_theta!(mock_theta_omega3, name, args, env),
        "mock_theta_nu3" => dispatch_mock_theta!(mock_theta_nu3, name, args, env),
        "mock_theta_rho3" => dispatch_mock_theta!(mock_theta_rho3, name, args, env),
        "mock_theta_f0_5" => dispatch_mock_theta!(mock_theta_f0_5, name, args, env),
        "mock_theta_f1_5" => dispatch_mock_theta!(mock_theta_f1_5, name, args, env),
        "mock_theta_cap_f0_5" => dispatch_mock_theta!(mock_theta_cap_f0_5, name, args, env),
        "mock_theta_cap_f1_5" => dispatch_mock_theta!(mock_theta_cap_f1_5, name, args, env),
        "mock_theta_phi0_5" => dispatch_mock_theta!(mock_theta_phi0_5, name, args, env),
        "mock_theta_phi1_5" => dispatch_mock_theta!(mock_theta_phi1_5, name, args, env),
        "mock_theta_psi0_5" => dispatch_mock_theta!(mock_theta_psi0_5, name, args, env),
        "mock_theta_psi1_5" => dispatch_mock_theta!(mock_theta_psi1_5, name, args, env),
        "mock_theta_chi0_5" => dispatch_mock_theta!(mock_theta_chi0_5, name, args, env),
        "mock_theta_chi1_5" => dispatch_mock_theta!(mock_theta_chi1_5, name, args, env),
        "mock_theta_cap_f0_7" => dispatch_mock_theta!(mock_theta_cap_f0_7, name, args, env),
        "mock_theta_cap_f1_7" => dispatch_mock_theta!(mock_theta_cap_f1_7, name, args, env),
        "mock_theta_cap_f2_7" => dispatch_mock_theta!(mock_theta_cap_f2_7, name, args, env),

        // Appell-Lerch (3 functions)

        "appell_lerch_m" => {
            // appell_lerch_m(a_pow, z_pow, order)
            expect_args(name, args, 3)?;
            let a_pow = extract_i64(name, args, 0)?;
            let z_pow = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::appell_lerch_m(a_pow, z_pow, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "universal_mock_theta_g2" => {
            // g2(a_pow, order)
            expect_args(name, args, 2)?;
            let a_pow = extract_i64(name, args, 0)?;
            let order = extract_i64(name, args, 1)?;
            let result = qseries::universal_mock_theta_g2(a_pow, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "universal_mock_theta_g3" => {
            // g3(a_pow, order)
            expect_args(name, args, 2)?;
            let a_pow = extract_i64(name, args, 0)?;
            let order = extract_i64(name, args, 1)?;
            let result = qseries::universal_mock_theta_g3(a_pow, env.sym_q, order);
            Ok(Value::Series(result))
        }

        // Bailey (4 functions)

        "bailey_weak_lemma" => {
            // bailey_weak_lemma(pair_code, a_num, a_den, a_pow, max_n, order)
            // pair_code: 0=Unit, 1=RogersRamanujan, 2=QBinomial
            expect_args(name, args, 6)?;
            let pair_code = extract_i64(name, args, 0)?;
            let a = extract_monomial(name, args, 1)?;
            let max_n = extract_i64(name, args, 4)?;
            let order = extract_i64(name, args, 5)?;
            let db = qseries::BaileyDatabase::new();
            let pair = get_bailey_pair_by_code(name, &db, pair_code)?;
            let (lhs, rhs) = qseries::weak_bailey_lemma(&pair, &a, max_n, env.sym_q, order);
            Ok(Value::Pair(Box::new(Value::Series(lhs)), Box::new(Value::Series(rhs))))
        }

        "bailey_apply_lemma" => {
            // bailey_apply_lemma(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, max_n, order)
            expect_args(name, args, 12)?;
            let pair_code = extract_i64(name, args, 0)?;
            let a = extract_monomial(name, args, 1)?;
            let b = extract_monomial(name, args, 4)?;
            let c = extract_monomial(name, args, 7)?;
            let max_n = extract_i64(name, args, 10)?;
            let order = extract_i64(name, args, 11)?;
            let db = qseries::BaileyDatabase::new();
            let pair = get_bailey_pair_by_code(name, &db, pair_code)?;
            let result = qseries::bailey_lemma(&pair, &a, &b, &c, max_n, env.sym_q, order);
            Ok(bailey_pair_to_value(&result))
        }

        "bailey_chain" => {
            // bailey_chain(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, depth, max_n, order)
            expect_args(name, args, 13)?;
            let pair_code = extract_i64(name, args, 0)?;
            let a = extract_monomial(name, args, 1)?;
            let b = extract_monomial(name, args, 4)?;
            let c = extract_monomial(name, args, 7)?;
            let depth = extract_i64(name, args, 10)? as usize;
            let max_n = extract_i64(name, args, 11)?;
            let order = extract_i64(name, args, 12)?;
            let db = qseries::BaileyDatabase::new();
            let pair = get_bailey_pair_by_code(name, &db, pair_code)?;
            let chain = qseries::bailey_chain(&pair, &a, &b, &c, depth, max_n, env.sym_q, order);
            Ok(Value::List(chain.iter().map(|p| bailey_pair_to_value(p)).collect()))
        }

        "bailey_discover" => {
            // bailey_discover(lhs, rhs, a_num, a_den, a_pow, max_depth, order)
            expect_args(name, args, 7)?;
            let lhs = extract_series(name, args, 0)?;
            let rhs = extract_series(name, args, 1)?;
            let a = extract_monomial(name, args, 2)?;
            let max_depth = extract_i64(name, args, 5)? as usize;
            let order = extract_i64(name, args, 6)?;
            let db = qseries::BaileyDatabase::new();
            let result = qseries::bailey_discover(&lhs, &rhs, &db, &a, max_depth, env.sym_q, order);
            Ok(discovery_result_to_value(&result))
        }

        // =================================================================
        // Group 8: Identity Proving (FUNC-08) -- 8 functions
        // =================================================================

        "prove_eta_id" => {
            // prove_eta_id(terms_list, level)
            // terms_list: [[coeff_n, coeff_d, [[delta1, exp1], [delta2, exp2], ...]], ...]
            expect_args(name, args, 2)?;
            let identity = extract_eta_identity(name, args)?;
            let result = qseries::prove_eta_identity(&identity);
            Ok(proof_result_to_value(&result))
        }

        "search_identities" => {
            // search_identities(search_type, query_code)
            // search_type: 0=all entries, 1=by_tag(code), 2=by_function(code)
            // Without string support, returns all entries for type 0.
            expect_args(name, args, 1)?;
            let search_type = extract_i64(name, args, 0)?;
            let db = qseries::IdentityDatabase::new();
            let tag_map: &[&str] = &[
                "classical", "partition", "theta", "eta", "mock_theta",
                "ramanujan", "jacobi", "euler",
            ];
            match search_type {
                0 => {
                    // Return all entries
                    Ok(Value::List(
                        db.entries().iter().map(|e| identity_entry_to_value(e)).collect(),
                    ))
                }
                code if code >= 1 && (code as usize - 1) < tag_map.len() => {
                    let tag = tag_map[(code - 1) as usize];
                    let results = db.search_by_tag(tag);
                    Ok(Value::List(
                        results.iter().map(|e| identity_entry_to_value(e)).collect(),
                    ))
                }
                _ => {
                    Ok(Value::List(vec![]))
                }
            }
        }

        // Algorithmic summation (Pattern K)

        "q_gosper" => {
            // q_gosper(upper_list, lower_list, z_num, z_den, z_pow, q_num, q_den)
            expect_args(name, args, 7)?;
            let upper = extract_monomial_list(name, args, 0)?;
            let lower = extract_monomial_list(name, args, 1)?;
            let z_num = extract_i64(name, args, 2)?;
            let z_den = extract_i64(name, args, 3)?;
            let z_pow = extract_i64(name, args, 4)?;
            let q_num = extract_i64(name, args, 5)?;
            let q_den = extract_i64(name, args, 6)?;
            let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);
            let series = HypergeometricSeries { upper, lower, argument: z };
            let q_val = QRat::from((q_num, q_den));
            let result = qseries::q_gosper(&series, &q_val);
            Ok(q_gosper_result_to_value(&result))
        }

        "q_zeilberger" => {
            // q_zeilberger(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order)
            expect_args(name, args, 9)?;
            let upper = extract_monomial_list(name, args, 0)?;
            let lower = extract_monomial_list(name, args, 1)?;
            let z_num = extract_i64(name, args, 2)?;
            let z_den = extract_i64(name, args, 3)?;
            let z_pow = extract_i64(name, args, 4)?;
            let n = extract_i64(name, args, 5)?;
            let q_num = extract_i64(name, args, 6)?;
            let q_den = extract_i64(name, args, 7)?;
            let max_order = extract_i64(name, args, 8)? as usize;
            let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);
            let series = HypergeometricSeries { upper, lower, argument: z };
            let q_val = QRat::from((q_num, q_den));
            let (n_param_indices, n_is_in_argument) = qseries::detect_n_params(&series, n, &q_val);
            let result = qseries::q_zeilberger(&series, n, &q_val, max_order, &n_param_indices, n_is_in_argument);
            Ok(q_zeilberger_result_to_value(&result))
        }

        "verify_wz" => {
            // verify_wz requires the recurrence result from q_zeilberger, which is complex.
            // For simplicity: verify_wz(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order, max_k)
            expect_args(name, args, 10)?;
            let upper = extract_monomial_list(name, args, 0)?;
            let lower = extract_monomial_list(name, args, 1)?;
            let z_num = extract_i64(name, args, 2)?;
            let z_den = extract_i64(name, args, 3)?;
            let z_pow = extract_i64(name, args, 4)?;
            let n = extract_i64(name, args, 5)?;
            let q_num = extract_i64(name, args, 6)?;
            let q_den = extract_i64(name, args, 7)?;
            let max_order = extract_i64(name, args, 8)? as usize;
            let max_k = extract_i64(name, args, 9)? as usize;
            let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);
            let series = HypergeometricSeries { upper, lower, argument: z };
            let q_val = QRat::from((q_num, q_den));
            let (n_param_indices, n_is_in_argument) = qseries::detect_n_params(&series, n, &q_val);
            // First run Zeilberger to get the recurrence + certificate
            let zresult = qseries::q_zeilberger(&series, n, &q_val, max_order, &n_param_indices, n_is_in_argument);
            match zresult {
                qseries::QZeilbergerResult::Recurrence(ref zr) => {
                    let verified = qseries::verify_wz_certificate(
                        &series, n, &q_val, &zr.coefficients, &zr.certificate,
                        &n_param_indices, n_is_in_argument, max_k,
                    );
                    Ok(Value::Dict(vec![
                        ("verified".to_string(), Value::Bool(verified)),
                        ("recurrence".to_string(), q_zeilberger_result_to_value(&zresult)),
                    ]))
                }
                qseries::QZeilbergerResult::NoRecurrence => {
                    Ok(Value::Dict(vec![
                        ("verified".to_string(), Value::Bool(false)),
                        ("reason".to_string(), Value::List(
                            "no recurrence found".chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
                        )),
                    ]))
                }
            }
        }

        "q_petkovsek" => {
            // q_petkovsek(coeff_list, q_num, q_den)
            expect_args(name, args, 3)?;
            let coefficients = extract_qrat_list(name, args, 0)?;
            let q_num = extract_i64(name, args, 1)?;
            let q_den = extract_i64(name, args, 2)?;
            let q_val = QRat::from((q_num, q_den));
            let results = qseries::q_petkovsek(&coefficients, &q_val);
            Ok(Value::List(
                results.iter().map(|r| q_petkovsek_result_to_value(r)).collect(),
            ))
        }

        // Nonterminating (Pattern L)

        "prove_nonterminating" => {
            Err(EvalError::Other(
                "prove_nonterminating requires closure arguments; use the Python API for this function".to_string(),
            ))
        }

        // =================================================================
        // Variable management (SYM-03)
        // =================================================================

        "anames" => {
            expect_args(name, args, 0)?;
            let mut names: Vec<String> = env.variables.keys().cloned().collect();
            names.sort();
            Ok(Value::List(names.into_iter().map(Value::String).collect()))
        }

        "restart" => {
            expect_args(name, args, 0)?;
            env.reset();
            Ok(Value::String("Restart.".to_string()))
        }

        // =================================================================
        // Script loading (EXEC-06)
        // =================================================================

        "read" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::String(path) => {
                    match crate::script::execute_file(path, env, false) {
                        crate::script::ScriptResult::Success => Ok(Value::None),
                        crate::script::ScriptResult::ParseError(msg) => {
                            Err(EvalError::Other(msg))
                        }
                        crate::script::ScriptResult::EvalError(msg) => {
                            Err(EvalError::Other(msg))
                        }
                        crate::script::ScriptResult::Panic(msg) => {
                            Err(EvalError::Panic(msg))
                        }
                        crate::script::ScriptResult::FileNotFound(msg) => {
                            Err(EvalError::Other(msg))
                        }
                        crate::script::ScriptResult::IoError(msg) => {
                            Err(EvalError::Other(msg))
                        }
                    }
                }
                _ => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "string",
                    got: args[0].type_name().to_string(),
                }),
            }
        }

        // =================================================================
        // Jacobi Product Constructor (NEW-01/02/03)
        // =================================================================

        "jac" | "JAC" => {
            expect_args(name, args, 2)?;
            let a = extract_i64(name, args, 0)?;
            let b = extract_i64(name, args, 1)?;
            if b <= 0 {
                return Err(EvalError::Other(format!(
                    "JAC: second argument (b) must be a positive integer, got {}", b
                )));
            }
            Ok(Value::JacobiProduct(vec![(a, b, 1)]))
        }

        "theta" => {
            expect_args(name, args, 3)?;
            let sym = extract_symbol_id(name, args, 1, env)?;
            let t_range = extract_i64(name, args, 2)?;

            match &args[0] {
                // Case 1: z is numeric (Integer or Rational)
                Value::Integer(_) | Value::Rational(_) => {
                    let z_val = extract_qrat(name, args, 0)?;
                    let mut fps = FormalPowerSeries::zero(sym, t_range);
                    for i in -t_range..=t_range {
                        let q_exp = i * i;
                        if q_exp >= t_range { continue; }
                        let z_pow_i = qrat_pow(&z_val, i);
                        let old = fps.coeff(q_exp);
                        fps.set_coeff(q_exp, old + z_pow_i);
                    }
                    Ok(Value::Series(fps))
                }
                // Case 2: z is a q-monomial (Series)
                Value::Series(_) => {
                    let mono = extract_monomial_from_arg(name, args, 0)?;
                    let mono_power = mono.power;
                    let mono_coeff = mono.coeff;
                    let mut fps = FormalPowerSeries::zero(sym, t_range);
                    for i in -t_range..=t_range {
                        let q_exp = mono_power * i + i * i;
                        if q_exp < 0 || q_exp >= t_range { continue; }
                        let coeff_i = qrat_pow(&mono_coeff, i);
                        let old = fps.coeff(q_exp);
                        fps.set_coeff(q_exp, old + coeff_i);
                    }
                    Ok(Value::Series(fps))
                }
                // Case 3: z is a bare Symbol -> warn, don't error
                Value::Symbol(sym_name) => {
                    println!("Warning: theta(z, q, T) requires z to be numeric or a q-monomial; '{}' is an unassigned symbol", sym_name);
                    Ok(Value::None)
                }
                _ => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "numeric value, q-monomial, or symbol",
                    got: args[0].type_name().to_string(),
                })
            }
        }

        "jac2prod" => {
            expect_args(name, args, 3)?;
            let factors = match &args[0] {
                Value::JacobiProduct(f) => f.clone(),
                _ => return Err(EvalError::Other(
                    "expected Jacobi product expression (use JAC(a,b))".to_string()
                )),
            };
            let sym = extract_symbol_id(name, args, 1, env)?;
            let order = extract_i64(name, args, 2)?;
            let sym_name = env.symbols.name(sym);

            // Print product notation
            let notation = format_product_notation(&factors, sym_name, order);
            println!("{}", notation);

            // Compute and return FPS
            let fps = jacobi_product_to_fps(&factors, sym, order);
            Ok(Value::Series(fps))
        }

        "jac2series" => {
            if args.len() == 2 {
                // Garvan 2-arg: jac2series(jacexpr, T)
                let factors = match &args[0] {
                    Value::JacobiProduct(f) => f.clone(),
                    _ => return Err(EvalError::Other(
                        "expected Jacobi product expression (use JAC(a,b))".to_string()
                    )),
                };
                let order = extract_i64(name, args, 1)?;
                let fps = jacobi_product_to_fps_garvan(&factors, env.sym_q, order);
                let formatted = crate::format::format_value(&Value::Series(fps.clone()), &env.symbols);
                println!("{}", formatted);
                Ok(Value::Series(fps))
            } else if args.len() == 3 {
                // Legacy 3-arg: jac2series(JP, q, T)
                let factors = match &args[0] {
                    Value::JacobiProduct(f) => f.clone(),
                    _ => return Err(EvalError::Other(
                        "expected Jacobi product expression (use JAC(a,b))".to_string()
                    )),
                };
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let fps = jacobi_product_to_fps(&factors, sym, order);
                let formatted = crate::format::format_value(&Value::Series(fps.clone()), &env.symbols);
                println!("{}", formatted);
                Ok(Value::Series(fps))
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "2 or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        "qs2jaccombo" => {
            expect_args(name, args, 3)?;
            let f = extract_series(name, args, 0)?;
            let sym = extract_symbol_id(name, args, 1, env)?;
            let order = extract_i64(name, args, 2)?;

            // Phase A: Try single JAC product via jacprodmake
            let jpform = qseries::jacprodmake(&f, order);
            if jpform.is_exact {
                let mut factors: Vec<(i64, i64, i64)> = jpform.factors.iter()
                    .map(|(&(a, b), &exp)| (a, b, exp))
                    .collect();
                factors.sort_by_key(|&(a, b, _)| (b, a));
                let jp_str = format_jacobi_product_value(&factors);
                let result_str = if jpform.scalar == QRat::from((1i64, 1i64)) {
                    jp_str
                } else {
                    format!("{}*{}", jpform.scalar, jp_str)
                };
                println!("{}", result_str);
                return Ok(Value::String(result_str));
            }

            // Phase B: Generate candidate JAC basis from identified periods
            let mut periods: Vec<i64> = jpform.factors.keys().map(|&(_, b)| b).collect();
            periods.sort();
            periods.dedup();

            // If no periods found, try small periods 2..min(order, 20)
            if periods.is_empty() {
                periods = (2..=std::cmp::min(order, 20)).collect();
            }

            // Generate candidate (a,b) pairs and expand each to FPS
            let mut candidate_labels: Vec<String> = Vec::new();
            let mut candidate_fps: Vec<FormalPowerSeries> = Vec::new();

            for &b in &periods {
                for a in 1..b {
                    let fps = qseries::etaq(a, b, sym, order);
                    candidate_labels.push(format!("JAC({},{})", a, b));
                    candidate_fps.push(fps);
                }
            }

            if candidate_fps.is_empty() {
                println!("No Jacobi product decomposition found");
                return Ok(Value::Series(f));
            }

            // Build references for findlincombo
            let refs: Vec<&FormalPowerSeries> = candidate_fps.iter().collect();
            match qseries::findlincombo(&f, &refs, 0) {
                Some(coeffs) => {
                    let formula = format_linear_combo(&coeffs, &candidate_labels);
                    println!("{}", formula);
                    Ok(Value::String(formula))
                }
                None => {
                    println!("No Jacobi product decomposition found");
                    Ok(Value::Series(f))
                }
            }
        }

        // =================================================================
        // Expression Operations (SERIES-01, SERIES-02)
        // =================================================================

        "series" => {
            expect_args(name, args, 3)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let order = extract_i64(name, args, 2)?;

            if order <= 0 {
                return Ok(Value::Series(FormalPowerSeries::zero(env.sym_q, 0)));
            }

            match &args[0] {
                Value::Series(fps) => {
                    let effective_order = order.min(fps.truncation_order());
                    let new_coeffs: BTreeMap<i64, QRat> = fps.iter()
                        .filter(|(&k, _)| k < effective_order)
                        .map(|(&k, v)| (k, v.clone()))
                        .collect();
                    Ok(Value::Series(FormalPowerSeries::from_coeffs(
                        fps.variable(), new_coeffs, effective_order
                    )))
                }
                Value::JacobiProduct(factors) => {
                    let fps = jacobi_product_to_fps(factors, env.sym_q, order);
                    Ok(Value::Series(fps))
                }
                Value::Integer(n) => {
                    let mut coeffs = BTreeMap::new();
                    if !n.0.is_zero() {
                        coeffs.insert(0, QRat::from(n.clone()));
                    }
                    Ok(Value::Series(FormalPowerSeries::from_coeffs(
                        env.sym_q, coeffs, order
                    )))
                }
                Value::Rational(r) => {
                    let mut coeffs = BTreeMap::new();
                    if !r.0.numer().is_zero() {
                        coeffs.insert(0, r.clone());
                    }
                    Ok(Value::Series(FormalPowerSeries::from_coeffs(
                        env.sym_q, coeffs, order
                    )))
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "series, Jacobi product, integer, or rational",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "expand" => {
            expect_args_range(name, args, 1, 3)?;

            if args.len() == 1 {
                match &args[0] {
                    Value::Series(_) => Ok(args[0].clone()),
                    Value::JacobiProduct(factors) => {
                        let fps = jacobi_product_to_fps(factors, env.sym_q, env.default_order);
                        Ok(Value::Series(fps))
                    }
                    Value::Integer(_) | Value::Rational(_) => Ok(args[0].clone()),
                    other => Err(EvalError::ArgType {
                        function: name.to_string(),
                        arg_index: 0,
                        expected: "series, Jacobi product, integer, or rational",
                        got: other.type_name().to_string(),
                    }),
                }
            } else if args.len() == 3 {
                let _sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;

                match &args[0] {
                    Value::Series(_) => Ok(args[0].clone()),
                    Value::JacobiProduct(factors) => {
                        let fps = jacobi_product_to_fps(factors, env.sym_q, order);
                        Ok(Value::Series(fps))
                    }
                    other => Err(EvalError::ArgType {
                        function: name.to_string(),
                        arg_index: 0,
                        expected: "series or Jacobi product",
                        got: other.type_name().to_string(),
                    }),
                }
            } else {
                Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1 or 3".to_string(),
                    got: args.len(),
                    signature: get_signature(name),
                })
            }
        }

        // =================================================================
        // Number Theory (UTIL-01, UTIL-02)
        // =================================================================

        "floor" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Integer(_) => Ok(args[0].clone()),
                Value::Rational(r) => {
                    let result = rug::Integer::from(r.0.floor_ref());
                    Ok(Value::Integer(QInt(result)))
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "number (integer or rational)",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "legendre" => {
            expect_args(name, args, 2)?;
            let m = extract_i64(name, args, 0)?;
            let p = extract_i64(name, args, 1)?;
            if p < 3 || p % 2 == 0 {
                return Err(EvalError::Other(format!(
                    "legendre: second argument must be an odd prime >= 3, got {}", p
                )));
            }
            let m_int = rug::Integer::from(m);
            let p_int = rug::Integer::from(p);
            let result = m_int.legendre(&p_int);
            Ok(Value::Integer(QInt::from(result as i64)))
        }

        "min" => {
            if args.is_empty() {
                return Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1 or more".to_string(),
                    got: 0,
                    signature: get_signature(name),
                });
            }
            let mut min_idx = 0;
            let mut min_val = extract_qrat(name, args, 0)?;
            for i in 1..args.len() {
                let val = extract_qrat(name, args, i)?;
                if val < min_val {
                    min_val = val;
                    min_idx = i;
                }
            }
            Ok(args[min_idx].clone())
        }

        "max" => {
            if args.is_empty() {
                return Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1 or more".to_string(),
                    got: 0,
                    signature: get_signature(name),
                });
            }
            let mut max_idx = 0;
            let mut max_val = extract_qrat(name, args, 0)?;
            for i in 1..args.len() {
                let val = extract_qrat(name, args, i)?;
                if val > max_val {
                    max_val = val;
                    max_idx = i;
                }
            }
            Ok(args[max_idx].clone())
        }

        // =================================================================
        // Polynomial Operations (POLY-01)
        // =================================================================

        "factor" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Series(fps) => {
                    let qrp = fps_to_qratpoly(fps).map_err(EvalError::Other)?;
                    let factorization = qsym_core::poly::factor_over_q(&qrp);
                    let display = factorization.display_with_var("q");
                    Ok(Value::String(display))
                }
                Value::Integer(n) => {
                    if n.is_zero() {
                        return Err(EvalError::Other("cannot factor zero".to_string()));
                    }
                    let qrp = qsym_core::QRatPoly::from_vec(vec![QRat::from(n.clone())]);
                    let factorization = qsym_core::poly::factor_over_q(&qrp);
                    let display = factorization.display_with_var("q");
                    Ok(Value::String(display))
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "polynomial series or integer",
                    got: other.type_name().to_string(),
                }),
            }
        }

        // =================================================================
        // Simplification Functions
        // =================================================================

        "radsimp" => {
            expect_args(name, args, 1)?;
            // radsimp simplifies rational series expressions.
            // Since series division is already computed during evaluation,
            // radsimp acts as an identity function -- the simplification
            // already happened when the argument was evaluated.
            Ok(args[0].clone())
        }

        // =================================================================
        // List Operations
        // =================================================================

        "nops" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::List(items) => Ok(Value::Integer(QInt::from(items.len() as i64))),
                Value::Series(fps) => {
                    // Count nonzero terms (FPS stores only nonzero coefficients)
                    let count = fps.iter().count();
                    Ok(Value::Integer(QInt::from(count as i64)))
                }
                Value::Integer(_) | Value::Rational(_) => Ok(Value::Integer(QInt::from(1i64))),
                Value::Symbol(_) => Ok(Value::Integer(QInt::from(1i64))),
                Value::BivariateSeries(bvs) => {
                    // Count nonzero z-coefficients
                    let count = bvs.terms.iter().filter(|(_, fps)| !fps.is_zero()).count();
                    Ok(Value::Integer(QInt::from(count as i64)))
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "list, series, integer, rational, or symbol",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "op" => {
            expect_args(name, args, 2)?;
            let i = match &args[0] {
                Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::Other(
                    "op: index too large".to_string()
                ))?,
                _ => return Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "integer",
                    got: args[0].type_name().to_string(),
                }),
            };
            match &args[1] {
                Value::List(items) => {
                    if i < 1 || i as usize > items.len() {
                        return Err(EvalError::Other(format!(
                            "op: index {} out of range (expression has {} operands)",
                            i, items.len()
                        )));
                    }
                    Ok(items[(i - 1) as usize].clone())
                }
                Value::Series(fps) => {
                    // Return i-th nonzero term as [exponent, coefficient]
                    let nonzero: Vec<_> = fps.iter().collect();
                    if i < 1 || i as usize > nonzero.len() {
                        return Err(EvalError::Other(format!(
                            "op: index {} out of range (series has {} nonzero terms)",
                            i, nonzero.len()
                        )));
                    }
                    let (exp, coeff) = nonzero[(i - 1) as usize];
                    Ok(Value::List(vec![
                        Value::Integer(QInt::from(*exp)),
                        if *coeff.denom() == 1 {
                            Value::Integer(QInt(coeff.numer().clone()))
                        } else {
                            Value::Rational(coeff.clone())
                        },
                    ]))
                }
                Value::Integer(_) | Value::Rational(_) | Value::Symbol(_) => {
                    if i == 1 {
                        Ok(args[1].clone())
                    } else {
                        Err(EvalError::Other(format!(
                            "op: index {} out of range (expression has 1 operand)", i
                        )))
                    }
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 1,
                    expected: "list, series, integer, rational, or symbol",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "map" => {
            expect_args(name, args, 2)?;
            let func = args[0].clone();
            let list = match &args[1] {
                Value::List(items) => items.clone(),
                other => return Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 1,
                    expected: "list",
                    got: other.type_name().to_string(),
                }),
            };
            let mut result = Vec::with_capacity(list.len());
            for elem in &list {
                let val = match &func {
                    Value::Procedure(proc) => call_procedure(proc, &[elem.clone()], env)?,
                    Value::Symbol(fname) => dispatch(fname, &[elem.clone()], env)?,
                    other => return Err(EvalError::ArgType {
                        function: name.to_string(),
                        arg_index: 0,
                        expected: "procedure or function name",
                        got: other.type_name().to_string(),
                    }),
                };
                result.push(val);
            }
            Ok(Value::List(result))
        }

        "sort" => {
            expect_args(name, args, 1)?;
            let list = match &args[0] {
                Value::List(items) => items.clone(),
                other => return Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "list",
                    got: other.type_name().to_string(),
                }),
            };
            let mut sorted = list;
            let mut sort_error: Option<String> = None;
            sorted.sort_by(|a, b| {
                match compare_values_for_sort(a, b) {
                    Some(ord) => ord,
                    None => {
                        if sort_error.is_none() {
                            sort_error = Some(format!(
                                "sort: cannot compare {} with {}",
                                a.type_name(), b.type_name()
                            ));
                        }
                        std::cmp::Ordering::Equal
                    }
                }
            });
            if let Some(err) = sort_error {
                return Err(EvalError::Other(err));
            }
            Ok(Value::List(sorted))
        }

        // =================================================================
        // Series Coefficient & Utility Functions
        // =================================================================

        "coeff" => {
            expect_args(name, args, 3)?;
            match &args[0] {
                Value::Series(fps) => {
                    let _sym = extract_symbol_id(name, args, 1, env)?;
                    let n = extract_i64(name, args, 2)?;
                    if n >= fps.truncation_order() {
                        return Err(EvalError::Other(format!(
                            "coeff: q^{} is beyond truncation order O(q^{})",
                            n, fps.truncation_order()
                        )));
                    }
                    let c = fps.coeff(n);
                    if *c.denom() == 1 {
                        Ok(Value::Integer(QInt(c.numer().clone())))
                    } else {
                        Ok(Value::Rational(c))
                    }
                }
                Value::Integer(n_val) => {
                    let _sym = extract_symbol_id(name, args, 1, env)?;
                    let exp = extract_i64(name, args, 2)?;
                    if exp == 0 { Ok(Value::Integer(n_val.clone())) }
                    else { Ok(Value::Integer(QInt::from(0i64))) }
                }
                Value::Rational(r) => {
                    let _sym = extract_symbol_id(name, args, 1, env)?;
                    let exp = extract_i64(name, args, 2)?;
                    if exp == 0 { Ok(Value::Rational(r.clone())) }
                    else { Ok(Value::Integer(QInt::from(0i64))) }
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "series, integer, or rational",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "degree" => {
            expect_args(name, args, 2)?;
            let _sym = extract_symbol_id(name, args, 1, env)?;
            match &args[0] {
                Value::Series(fps) => {
                    match qseries::qdegree(fps) {
                        Some(d) => Ok(Value::Integer(QInt::from(d))),
                        None => Ok(Value::Integer(QInt::from(0i64))),
                    }
                }
                Value::Integer(_) | Value::Rational(_) => {
                    Ok(Value::Integer(QInt::from(0i64)))
                }
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "series, integer, or rational",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "numer" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Rational(r) => Ok(Value::Integer(QInt(r.numer().clone()))),
                Value::Integer(n) => Ok(Value::Integer(n.clone())),
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "rational or integer",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "denom" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Rational(r) => Ok(Value::Integer(QInt(r.denom().clone()))),
                Value::Integer(_) => Ok(Value::Integer(QInt::from(1i64))),
                other => Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 0,
                    expected: "rational or integer",
                    got: other.type_name().to_string(),
                }),
            }
        }

        "modp" => {
            expect_args(name, args, 2)?;
            let a = extract_i64(name, args, 0)?;
            let p = extract_i64(name, args, 1)?;
            if p <= 0 {
                return Err(EvalError::Other("modp: modulus must be positive".into()));
            }
            let result = ((a % p) + p) % p;
            Ok(Value::Integer(QInt::from(result)))
        }

        "mods" => {
            expect_args(name, args, 2)?;
            let a = extract_i64(name, args, 0)?;
            let p = extract_i64(name, args, 1)?;
            if p <= 0 {
                return Err(EvalError::Other("mods: modulus must be positive".into()));
            }
            let r = ((a % p) + p) % p;
            if r * 2 > p { Ok(Value::Integer(QInt::from(r - p))) }
            else { Ok(Value::Integer(QInt::from(r))) }
        }

        "type" => {
            expect_args(name, args, 2)?;
            let type_name_str = match &args[1] {
                Value::Symbol(s) => s.clone(),
                Value::String(s) => s.clone(),
                other => return Err(EvalError::ArgType {
                    function: name.to_string(),
                    arg_index: 1,
                    expected: "symbol or string (type name)",
                    got: other.type_name().to_string(),
                }),
            };
            let matches = match type_name_str.as_str() {
                "integer" => matches!(&args[0], Value::Integer(_)),
                "rational" => matches!(&args[0], Value::Rational(_)),
                "numeric" => matches!(&args[0], Value::Integer(_) | Value::Rational(_)),
                "series" => matches!(&args[0], Value::Series(_)),
                "list" => matches!(&args[0], Value::List(_)),
                "string" => matches!(&args[0], Value::String(_)),
                "boolean" => matches!(&args[0], Value::Bool(_)),
                "symbol" | "name" => matches!(&args[0], Value::Symbol(_)),
                "procedure" => matches!(&args[0], Value::Procedure(_)),
                "infinity" => matches!(&args[0], Value::Infinity),
                _ => false,
            };
            Ok(Value::Bool(matches))
        }

        "evalb" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::Integer(n) => Ok(Value::Bool(!n.is_zero())),
                other => Err(EvalError::Other(format!(
                    "evalb: expected boolean or integer, got {}", other.type_name()
                ))),
            }
        }

        "cat" => {
            if args.is_empty() {
                return Err(EvalError::WrongArgCount {
                    function: name.to_string(),
                    expected: "1+".to_string(),
                    got: 0,
                    signature: get_signature(name),
                });
            }
            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::Symbol(s) => result.push_str(s),
                    Value::String(s) => result.push_str(s),
                    Value::Integer(n) => result.push_str(&n.0.to_string()),
                    Value::Rational(r) => result.push_str(&format!("{}/{}", r.numer(), r.denom())),
                    Value::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
                    _ => result.push_str(arg.type_name()),
                }
            }
            Ok(Value::Symbol(result))
        }

        // =================================================================
        // Package Info (Maple parity)
        // =================================================================

        "changes" => {
            expect_args(name, args, 0)?;
            let text = "\
q-Kangaroo changelog:
  v5.0 (2026-02-22): while-loops, lists, add/mul/seq ranges, 121 functions
  v4.0 (2026-02-21): ditto, arrow lambdas, qmaple.pdf parity, 101 functions
  v3.0 (2026-02-21): for-loops, procedures, bivariate series
  v2.0 (2026-02-20): Maple-compatible function signatures, 89 functions
  v1.6 (2026-02-18): static GMP linking, script execution, PDF manual
  v1.5 (2026-02-18): interactive REPL, Pratt parser, 81 functions
  v1.2 (2026-02-16): q-Gosper, q-Zeilberger, WZ certificates
  v1.0 (2026-02-14): core engine, 73 functions, 578 tests";
            println!("{}", text);
            Ok(Value::String(text.to_string()))
        }

        "packageversion" => {
            expect_args(name, args, 0)?;
            let version = "q-Kangaroo v5.0 (2026-02-22)";
            println!("{}", version);
            Ok(Value::String(version.to_string()))
        }

        // =================================================================
        // Unknown function
        // =================================================================
        _ => {
            let suggestions = find_similar_names(&canonical);
            Err(EvalError::UnknownFunction {
                name: name.to_string(),
                suggestions,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Sorting helper
// ---------------------------------------------------------------------------

/// Compare two Values for sorting purposes.
/// Numeric types are compared by value (Integer and Rational unified via QRat).
/// Symbols and strings are compared lexicographically.
/// Returns None if types are incomparable.
fn compare_values_for_sort(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(x.0.cmp(&y.0)),
        (Value::Rational(x), Value::Rational(y)) => Some(x.cmp(y)),
        (Value::Integer(x), Value::Rational(y)) => {
            let xr = QRat::from(x.clone());
            Some(xr.cmp(y))
        }
        (Value::Rational(x), Value::Integer(y)) => {
            let yr = QRat::from(y.clone());
            Some(x.cmp(&yr))
        }
        (Value::Symbol(x), Value::Symbol(y)) => Some(x.cmp(y)),
        (Value::String(x), Value::String(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// subs() substitution logic
// ---------------------------------------------------------------------------

/// Perform variable substitution on a target value.
///
/// Supports:
/// - `subs(q=rational, Series)` -- evaluate polynomial at a rational point
/// - `subs(q=0, Series)` -- return constant term
/// - `subs(q=q^k, Series)` -- scale all exponents by k
/// - `subs(q=anything, non-Series)` -- return target unchanged (constant)
fn perform_substitution(
    var_name: &str,
    sub_value: Value,
    target: Value,
    env: &mut Environment,
) -> Result<Value, EvalError> {
    // For indexed variables (like X[1]), handle Symbol targets by matching
    if var_name.contains('[') {
        match &target {
            Value::Symbol(s) if *s == var_name => {
                return Ok(sub_value);
            }
            Value::String(s) if s.contains(var_name) => {
                let replaced = s.replace(var_name, &format!("({})", format_value_for_subs(&sub_value, env)));
                return Ok(Value::String(replaced));
            }
            _ => {}
        }
    }

    // For non-Series targets, substitution is a no-op (constant)
    let fps = match &target {
        Value::Series(fps) => fps,
        Value::Integer(_) | Value::Rational(_) | Value::Bool(_)
        | Value::String(_) | Value::None | Value::Infinity | Value::Symbol(_) => {
            return Ok(target);
        }
        _ => return Ok(target),
    };

    // Check that the variable name matches the series variable
    let series_var_name = env.symbols.name(fps.variable()).to_string();
    if series_var_name != var_name {
        // Variable doesn't match -- return target unchanged
        return Ok(target);
    }

    match sub_value {
        // Case: subs(q=integer, Series) -- evaluate at integer point
        Value::Integer(ref n) => {
            let rat = QRat::from(n.clone());
            evaluate_fps_at_rational(fps, &rat)
        }
        // Case: subs(q=rational, Series) -- evaluate at rational point
        Value::Rational(ref r) => {
            evaluate_fps_at_rational(fps, r)
        }
        // Case: subs(q=Series, Series) -- check for q^k pattern (exponent scaling)
        Value::Series(ref sub_fps) => {
            // Detect if sub_value is q^k for some positive integer k
            if sub_fps.variable() != fps.variable() {
                return Err(EvalError::Other(
                    "subs: substitution series must use the same variable".into()
                ));
            }
            let terms: Vec<_> = sub_fps.iter().collect();
            if terms.len() != 1 {
                return Err(EvalError::Other(
                    "subs: substitution value must be q^k for some positive integer k".into()
                ));
            }
            let (&exp, coeff) = terms[0];
            if *coeff != QRat::one() || exp <= 0 {
                return Err(EvalError::Other(
                    "subs: substitution value must be q^k for some positive integer k".into()
                ));
            }
            let k = exp;

            // Scale exponents: each (e, c) -> (e*k, c)
            let mut new_coeffs = BTreeMap::new();
            for (&e, c) in fps.iter() {
                new_coeffs.insert(e * k, c.clone());
            }

            // Scale truncation order
            let new_trunc = if fps.truncation_order() == POLYNOMIAL_ORDER {
                POLYNOMIAL_ORDER
            } else {
                fps.truncation_order() * k
            };

            Ok(Value::Series(FormalPowerSeries::from_coeffs(
                fps.variable(),
                new_coeffs,
                new_trunc,
            )))
        }
        _ => Err(EvalError::Other(
            "subs: substitution value must be a number or q^k expression".into()
        )),
    }
}

/// Format a Value for string-based substitution in subs().
fn format_value_for_subs(val: &Value, env: &Environment) -> String {
    crate::format::format_value(val, &env.symbols)
}

/// Evaluate a FPS at a rational point: sum c_k * val^k over all terms.
fn evaluate_fps_at_rational(
    fps: &FormalPowerSeries,
    val: &QRat,
) -> Result<Value, EvalError> {
    let mut result = QRat::zero();

    for (&exp, coeff) in fps.iter() {
        if exp == 0 {
            // c * val^0 = c
            result = &result + coeff;
        } else if exp > 0 {
            // c * val^exp
            let mut power = QRat::one();
            for _ in 0..exp {
                power = &power * val;
            }
            result = &result + &(coeff * &power);
        } else {
            // Negative exponent: c * val^exp = c / val^|exp|
            if val.is_zero() {
                return Err(EvalError::Other(format!(
                    "subs: cannot evaluate at 0 with negative exponent q^{}",
                    exp
                )));
            }
            let abs_exp = (-exp) as u64;
            let mut power = QRat::one();
            for _ in 0..abs_exp {
                power = &power * val;
            }
            let inv_power = &QRat::one() / &power;
            result = &result + &(coeff * &inv_power);
        }
    }

    // Return Integer if denominator is 1, otherwise Rational
    if result.denom() == &rug::Integer::from(1) {
        Ok(Value::Integer(QInt::from(rug::Integer::from(result.numer()))))
    } else {
        Ok(Value::Rational(result))
    }
}

// ---------------------------------------------------------------------------
// FPS -> QRatPoly conversion (for factor() dispatch)
// ---------------------------------------------------------------------------

/// Convert a FormalPowerSeries to a QRatPoly for polynomial operations.
///
/// Requires that the FPS has `POLYNOMIAL_ORDER` truncation (i.e., it's an exact
/// polynomial, not a truncated series). Returns `Err` with a user-facing message
/// if the conversion is not possible.
fn fps_to_qratpoly(fps: &FormalPowerSeries) -> Result<qsym_core::QRatPoly, String> {
    if fps.truncation_order() != POLYNOMIAL_ORDER {
        return Err(
            "cannot factor truncated series -- use expand() to get an exact polynomial".to_string(),
        );
    }
    if fps.is_zero() {
        return Err("cannot factor zero polynomial".to_string());
    }

    // Check for negative exponents
    if let Some((&min_exp, _)) = fps.iter().next() {
        if min_exp < 0 {
            return Err(format!(
                "polynomial has negative exponent q^{}",
                min_exp
            ));
        }
    }

    // Get max exponent
    let max_exp = fps
        .iter()
        .last()
        .map(|(&e, _)| e)
        .unwrap_or(0);

    // Build dense coefficient vector
    let len = (max_exp + 1) as usize;
    let mut coeffs = vec![QRat::zero(); len];
    for (&exp, coeff) in fps.iter() {
        coeffs[exp as usize] = coeff.clone();
    }

    Ok(qsym_core::QRatPoly::from_vec(coeffs))
}

// ---------------------------------------------------------------------------
// Conversion helpers (analysis result types -> Value::Dict)
// ---------------------------------------------------------------------------

/// Convert an `InfiniteProductForm` to `Value::Dict`.
fn infinite_product_form_to_value(ipf: &qseries::InfiniteProductForm) -> Value {
    let mut exp_entries: Vec<(String, Value)> = Vec::new();
    for (&n, r) in &ipf.exponents {
        exp_entries.push((n.to_string(), Value::Rational(r.clone())));
    }
    Value::Dict(vec![
        ("exponents".to_string(), Value::Dict(exp_entries)),
        ("terms_used".to_string(), Value::Integer(QInt::from(ipf.terms_used))),
    ])
}

/// Convert an `EtaQuotient` to `Value::Dict`.
fn eta_quotient_to_value(eq: &qseries::EtaQuotient) -> Value {
    Value::EtaQuotient {
        factors: eq.factors.clone(),
        q_shift: eq.q_shift.clone(),
    }
}

/// Convert a `JacobiProductForm` to `Value::Dict`.
fn jacobi_product_form_to_value(jpf: &qseries::JacobiProductForm) -> Value {
    let mut factor_entries: Vec<(String, Value)> = Vec::new();
    for (&(a, b), &exp) in &jpf.factors {
        factor_entries.push((
            format!("({},{})", a, b),
            Value::Integer(QInt::from(exp)),
        ));
    }
    Value::Dict(vec![
        ("factors".to_string(), Value::Dict(factor_entries)),
        ("scalar".to_string(), Value::Rational(jpf.scalar.clone())),
        ("is_exact".to_string(), Value::Bool(jpf.is_exact)),
    ])
}

/// Convert a `BTreeMap<i64, i64>` (mprodmake result) to `Value::Dict`.
fn btreemap_i64_to_value(m: &BTreeMap<i64, i64>) -> Value {
    let mut entries: Vec<(String, Value)> = Vec::new();
    for (&n, &exp) in m {
        entries.push((n.to_string(), Value::Integer(QInt::from(exp))));
    }
    Value::Dict(entries)
}

/// Convert a `QEtaForm` to `Value::Dict`.
fn q_eta_form_to_value(qef: &qseries::QEtaForm) -> Value {
    let mut factor_entries: Vec<(String, Value)> = Vec::new();
    for (&d, &r_d) in &qef.factors {
        factor_entries.push((d.to_string(), Value::Integer(QInt::from(r_d))));
    }
    Value::Dict(vec![
        ("factors".to_string(), Value::Dict(factor_entries)),
        ("q_shift".to_string(), Value::Rational(qef.q_shift.clone())),
    ])
}

/// Convert a `QFactorization` to `Value::QProduct`.
fn q_factorization_to_value(qf: &qseries::QFactorization) -> Value {
    Value::QProduct {
        factors: qf.factors.clone(),
        scalar: qf.scalar.clone(),
        is_exact: qf.is_exact,
    }
}

/// Convert a `Congruence` to `Value::Dict`.
fn congruence_to_value(c: &qseries::Congruence) -> Value {
    Value::Dict(vec![
        ("modulus".to_string(), Value::Integer(QInt::from(c.modulus_m))),
        ("residue".to_string(), Value::Integer(QInt::from(c.residue_b))),
        ("divisor".to_string(), Value::Integer(QInt::from(c.divisor_r))),
    ])
}

/// Convert a `PolynomialRelation` to `Value::Dict`.
fn polynomial_relation_to_value(rel: &qseries::PolynomialRelation) -> Value {
    let coeffs = Value::List(
        rel.coefficients.iter()
            .map(|row| Value::List(row.iter().map(|r| Value::Rational(r.clone())).collect()))
            .collect(),
    );
    Value::Dict(vec![
        ("coefficients".to_string(), coeffs),
        ("deg_x".to_string(), Value::Integer(QInt::from(rel.deg_x))),
        ("deg_y".to_string(), Value::Integer(QInt::from(rel.deg_y))),
    ])
}

/// Convert a `TransformationChainResult` to `Value::Dict`.
fn transformation_chain_result_to_value(r: &qseries::TransformationChainResult) -> Value {
    match r {
        qseries::TransformationChainResult::Found { steps, total_prefactor } => {
            let step_list = Value::List(
                steps.iter()
                    .map(|s| Value::Dict(vec![
                        ("name".to_string(), Value::List(
                            s.name.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
                        )),
                        ("prefactor".to_string(), Value::Series(s.step_prefactor.clone())),
                    ]))
                    .collect(),
            );
            Value::Dict(vec![
                ("found".to_string(), Value::Bool(true)),
                ("steps".to_string(), step_list),
                ("total_prefactor".to_string(), Value::Series(total_prefactor.clone())),
                ("depth".to_string(), Value::Integer(QInt::from(steps.len() as i64))),
            ])
        }
        qseries::TransformationChainResult::NotFound { max_depth } => {
            Value::Dict(vec![
                ("found".to_string(), Value::Bool(false)),
                ("max_depth".to_string(), Value::Integer(QInt::from(*max_depth as i64))),
            ])
        }
    }
}

/// Look up a Bailey pair from the database by integer code.
///
/// Codes: 0=Unit, 1=RogersRamanujan, 2=QBinomial.
fn get_bailey_pair_by_code(
    name: &str,
    db: &qseries::BaileyDatabase,
    code: i64,
) -> Result<qseries::BaileyPair, EvalError> {
    let pair_name = match code {
        0 => "unit",
        1 => "rogers-ramanujan",
        2 => "q-binomial",
        _ => {
            return Err(EvalError::ArgType {
                function: name.to_string(),
                arg_index: 0,
                expected: "pair code (0=Unit, 1=RogersRamanujan, 2=QBinomial)",
                got: format!("{}", code),
            });
        }
    };
    let results = db.search_by_name(pair_name);
    results.first()
        .map(|p| (*p).clone())
        .ok_or_else(|| EvalError::Other(format!("{}: pair '{}' not found in database", name, pair_name)))
}

/// Extract an `EtaIdentity` from args: (terms_list, level).
///
/// terms_list: `[[coeff_n, coeff_d, [[delta1, exp1], [delta2, exp2], ...]], ...]`
fn extract_eta_identity(name: &str, args: &[Value]) -> Result<qseries::EtaIdentity, EvalError> {
    let level = extract_i64(name, args, 1)?;
    match &args[0] {
        Value::List(terms) => {
            let mut result = Vec::with_capacity(terms.len());
            for (i, term) in terms.iter().enumerate() {
                match term {
                    Value::List(inner) if inner.len() == 3 => {
                        // inner = [coeff_num, coeff_den, [[delta, exp], ...]]
                        let cn = match &inner[0] {
                            Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: 0,
                                expected: "eta identity terms",
                                got: format!("integer too large in term {}", i),
                            })?,
                            other => return Err(EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: 0,
                                expected: "eta identity terms",
                                got: format!("{} in term {} position 0", other.type_name(), i),
                            }),
                        };
                        let cd = match &inner[1] {
                            Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: 0,
                                expected: "eta identity terms",
                                got: format!("integer too large in term {}", i),
                            })?,
                            other => return Err(EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: 0,
                                expected: "eta identity terms",
                                got: format!("{} in term {} position 1", other.type_name(), i),
                            }),
                        };
                        let coeff = QRat::from((cn, cd));
                        let factors_list = match &inner[2] {
                            Value::List(fl) => fl,
                            other => return Err(EvalError::ArgType {
                                function: name.to_string(),
                                arg_index: 0,
                                expected: "list of [delta, exp] pairs",
                                got: format!("{} in term {} position 2", other.type_name(), i),
                            }),
                        };
                        let mut factors = std::collections::BTreeMap::new();
                        for (j, factor) in factors_list.iter().enumerate() {
                            match factor {
                                Value::List(pair) if pair.len() == 2 => {
                                    let delta = match &pair[0] {
                                        Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                            function: name.to_string(),
                                            arg_index: 0,
                                            expected: "eta identity terms",
                                            got: format!("integer too large in factor ({},{})", i, j),
                                        })?,
                                        other => return Err(EvalError::ArgType {
                                            function: name.to_string(),
                                            arg_index: 0,
                                            expected: "eta identity terms",
                                            got: format!("{} in factor ({},{}) position 0", other.type_name(), i, j),
                                        }),
                                    };
                                    let exp = match &pair[1] {
                                        Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                                            function: name.to_string(),
                                            arg_index: 0,
                                            expected: "eta identity terms",
                                            got: format!("integer too large in factor ({},{})", i, j),
                                        })?,
                                        other => return Err(EvalError::ArgType {
                                            function: name.to_string(),
                                            arg_index: 0,
                                            expected: "eta identity terms",
                                            got: format!("{} in factor ({},{}) position 1", other.type_name(), i, j),
                                        }),
                                    };
                                    factors.insert(delta, exp);
                                }
                                other => return Err(EvalError::ArgType {
                                    function: name.to_string(),
                                    arg_index: 0,
                                    expected: "list of [delta, exp] pairs",
                                    got: format!("{} in factors ({},{})", other.type_name(), i, j),
                                }),
                            }
                        }
                        let eta_expr = qseries::EtaExpression::new(factors, level);
                        result.push((coeff, eta_expr));
                    }
                    other => return Err(EvalError::ArgType {
                        function: name.to_string(),
                        arg_index: 0,
                        expected: "list of [coeff_n, coeff_d, [[delta, exp], ...]] terms",
                        got: format!("{} at position {}", other.type_name(), i),
                    }),
                }
            }
            Ok(qseries::EtaIdentity::new(result, level))
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 0,
            expected: "list of eta identity terms",
            got: other.type_name().to_string(),
        }),
    }
}

/// Convert a `QGosperResult` to `Value::Dict`.
fn q_gosper_result_to_value(r: &qseries::QGosperResult) -> Value {
    match r {
        qseries::QGosperResult::Summable { certificate } => {
            let n_deg = match certificate.numer.degree() {
                Some(d) => Value::Integer(QInt::from(d as i64)),
                None => Value::None,
            };
            let d_deg = match certificate.denom.degree() {
                Some(d) => Value::Integer(QInt::from(d as i64)),
                None => Value::None,
            };
            Value::Dict(vec![
                ("summable".to_string(), Value::Bool(true)),
                ("certificate_numer_degree".to_string(), n_deg),
                ("certificate_denom_degree".to_string(), d_deg),
            ])
        }
        qseries::QGosperResult::NotSummable => {
            Value::Dict(vec![
                ("summable".to_string(), Value::Bool(false)),
            ])
        }
    }
}

/// Convert a `QZeilbergerResult` to `Value::Dict`.
fn q_zeilberger_result_to_value(r: &qseries::QZeilbergerResult) -> Value {
    match r {
        qseries::QZeilbergerResult::Recurrence(zr) => {
            Value::Dict(vec![
                ("found".to_string(), Value::Bool(true)),
                ("order".to_string(), Value::Integer(QInt::from(zr.order as i64))),
                ("coefficients".to_string(), Value::List(
                    zr.coefficients.iter().map(|c| Value::Rational(c.clone())).collect(),
                )),
            ])
        }
        qseries::QZeilbergerResult::NoRecurrence => {
            Value::Dict(vec![
                ("found".to_string(), Value::Bool(false)),
            ])
        }
    }
}

/// Convert a `QPetkovsekResult` to `Value::Dict`.
fn q_petkovsek_result_to_value(r: &qseries::QPetkovsekResult) -> Value {
    let mut entries = vec![
        ("ratio".to_string(), Value::Rational(r.ratio.clone())),
    ];
    if let Some(ref cf) = r.closed_form {
        entries.push(("has_closed_form".to_string(), Value::Bool(true)));
        entries.push(("scalar".to_string(), Value::Rational(cf.scalar.clone())));
        entries.push(("q_power_coeff".to_string(), Value::Integer(QInt::from(cf.q_power_coeff))));
        entries.push(("numer_factors".to_string(), Value::Integer(QInt::from(cf.numer_factors.len() as i64))));
        entries.push(("denom_factors".to_string(), Value::Integer(QInt::from(cf.denom_factors.len() as i64))));
    } else {
        entries.push(("has_closed_form".to_string(), Value::Bool(false)));
    }
    Value::Dict(entries)
}

/// Convert a `ProofResult` to `Value::Dict`.
fn proof_result_to_value(r: &qseries::ProofResult) -> Value {
    match r {
        qseries::ProofResult::Proved { level, cusp_orders, sturm_bound, verification_terms } => {
            let cusps = Value::List(
                cusp_orders.iter()
                    .map(|(c, ord)| Value::Dict(vec![
                        ("cusp".to_string(), Value::List(vec![
                            Value::Integer(QInt::from(c.numer)),
                            Value::Integer(QInt::from(c.denom)),
                        ])),
                        ("order".to_string(), Value::Rational(ord.clone())),
                    ]))
                    .collect(),
            );
            Value::Dict(vec![
                ("proved".to_string(), Value::Bool(true)),
                ("level".to_string(), Value::Integer(QInt::from(*level))),
                ("cusp_orders".to_string(), cusps),
                ("sturm_bound".to_string(), Value::Integer(QInt::from(*sturm_bound))),
                ("verification_terms".to_string(), Value::Integer(QInt::from(*verification_terms))),
            ])
        }
        qseries::ProofResult::NotModular { failed_conditions } => {
            Value::Dict(vec![
                ("proved".to_string(), Value::Bool(false)),
                ("reason".to_string(), Value::List(
                    failed_conditions.iter()
                        .map(|s| Value::List(s.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect()))
                        .collect(),
                )),
            ])
        }
        qseries::ProofResult::NegativeOrder { cusp, order } => {
            Value::Dict(vec![
                ("proved".to_string(), Value::Bool(false)),
                ("negative_order_cusp".to_string(), Value::List(vec![
                    Value::Integer(QInt::from(cusp.numer)),
                    Value::Integer(QInt::from(cusp.denom)),
                ])),
                ("order".to_string(), Value::Rational(order.clone())),
            ])
        }
        qseries::ProofResult::CounterExample { coefficient_index, expected, actual } => {
            Value::Dict(vec![
                ("proved".to_string(), Value::Bool(false)),
                ("counter_example_at".to_string(), Value::Integer(QInt::from(*coefficient_index))),
                ("expected".to_string(), Value::Rational(expected.clone())),
                ("actual".to_string(), Value::Rational(actual.clone())),
            ])
        }
    }
}

/// Convert a `DiscoveryResult` to `Value::Dict`.
fn discovery_result_to_value(r: &qseries::DiscoveryResult) -> Value {
    let mut entries = vec![
        ("found".to_string(), Value::Bool(r.found)),
        ("chain_depth".to_string(), Value::Integer(QInt::from(r.chain_depth as i64))),
    ];
    if let Some(ref pn) = r.pair_name {
        entries.push(("pair_name".to_string(), Value::List(
            pn.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
        )));
    }
    entries.push(("verification".to_string(), Value::List(
        r.verification.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
    )));
    Value::Dict(entries)
}

/// Convert a `BaileyPair` to `Value::Dict` (summary only).
fn bailey_pair_to_value(p: &qseries::BaileyPair) -> Value {
    Value::Dict(vec![
        ("name".to_string(), Value::List(
            p.name.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
        )),
        ("tags".to_string(), Value::List(
            p.tags.iter().map(|t| Value::List(
                t.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
            )).collect(),
        )),
    ])
}

/// Convert an `IdentityEntry` to `Value::Dict`.
fn identity_entry_to_value(e: &qseries::IdentityEntry) -> Value {
    Value::Dict(vec![
        ("id".to_string(), Value::List(
            e.id.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
        )),
        ("name".to_string(), Value::List(
            e.name.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
        )),
        ("tags".to_string(), Value::List(
            e.tags.iter().map(|t| Value::List(
                t.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
            )).collect(),
        )),
        ("functions".to_string(), Value::List(
            e.functions.iter().map(|f| Value::List(
                f.chars().map(|c| Value::Integer(QInt::from(c as i64))).collect(),
            )).collect(),
        )),
    ])
}

// ---------------------------------------------------------------------------
// Function signatures for error messages
// ---------------------------------------------------------------------------

/// Return the human-readable signature for a function, used in error messages.
fn get_signature(name: &str) -> String {
    match name {
        // Group 1: q-Pochhammer and Products
        "aqprod" => "(coeff_num, coeff_den, power, n_or_infinity, order) or (monomial, var, n[, order])".to_string(),
        "qbin" => "(q, m, n) or (n, k, q, T) or (n, k, order)".to_string(),
        "etaq" => "(q, delta, T) or (q, [deltas], T) or (b, t, order)".to_string(),
        "jacprod" => "(a, b, q, T) or (a, b, order)".to_string(),
        "tripleprod" => "(z, q, T) or (coeff_num, coeff_den, power, order)".to_string(),
        "quinprod" => "(z, q, T) or (z, q, prodid) or (z, q, seriesid) or (coeff_num, coeff_den, power, order)".to_string(),
        "winquist" => "(a, b, q, T) or (a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)".to_string(),
        // Group 2: Partitions
        "numbpart" => "(n) or (n, m)".to_string(),
        "partition_gf" => "(order)".to_string(),
        "distinct_parts_gf" => "(order)".to_string(),
        "odd_parts_gf" => "(order)".to_string(),
        "bounded_parts_gf" => "(max_part, order)".to_string(),
        "rank_gf" => "(z_num, z_den, order)".to_string(),
        "crank_gf" => "(z_num, z_den, order)".to_string(),
        // Group 3: Theta Functions
        "theta2" => "(T) or (q, T) or (a, q, T)".to_string(),
        "theta3" => "(T) or (q, T) or (a, q, T)".to_string(),
        "theta4" => "(T) or (q, T) or (a, q, T)".to_string(),
        // Group 4: Series Analysis
        "sift" => "(s, q, n, k, T)".to_string(),
        "qdegree" => "(series)".to_string(),
        "lqdegree" => "(series)".to_string(),
        "lqdegree0" => "(f)".to_string(),
        "checkmult" => "(QS, T) or (QS, T, 'yes')".to_string(),
        "checkprod" => "(f, M, Q)".to_string(),
        "prodmake" => "(f, q, T)".to_string(),
        "etamake" => "(f, q, T)".to_string(),
        "jacprodmake" => "(f, q, T) or (f, q, T, P)".to_string(),
        "mprodmake" => "(f, q, T)".to_string(),
        "qetamake" => "(f, q, T)".to_string(),
        "qfactor" => "(f, q) or (f, T) or (f, q, T)".to_string(),
        "zqfactor" => "(f, z, q) or (f, z, q, maxdeg)".to_string(),
        // Group 5: Relation Discovery
        "findlincombo" => "(f, L, SL, q, topshift)".to_string(),
        "findhomcombo" => "(f, L, q, n, topshift)".to_string(),
        "findnonhomcombo" => "(f, L, q, n, topshift)".to_string(),
        "findlincombomodp" => "(f, L, SL, p, q, topshift)".to_string(),
        "findhomcombomodp" => "(f, L, p, q, n, topshift)".to_string(),
        "findhom" => "(L, q, n, topshift)".to_string(),
        "findnonhom" => "(L, q, n, topshift)".to_string(),
        "findhommodp" => "(L, p, q, n, topshift)".to_string(),
        "findmaxind" => "(L, T)".to_string(),
        "findprod" => "(FL, T, M, Q)".to_string(),
        "findcong" => "(QS, T) or (QS, T, LM) or (QS, T, LM, XSET)".to_string(),
        "findpoly" => "(x, y, q, dx, dy) or (x, y, q, dx, dy, check)".to_string(),
        // Group 6: Hypergeometric
        "phi" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "psi" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "try_summation" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "heine1" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "heine2" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "heine3" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "sears_transform" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "watson_transform" => "(upper_list, lower_list, z_num, z_den, z_pow, order)".to_string(),
        "find_transformation_chain" => "(src_upper, src_lower, src_z_n, src_z_d, src_z_p, tgt_upper, tgt_lower, tgt_z_n, tgt_z_d, tgt_z_p, max_depth, order)".to_string(),
        // Group 7: Mock Theta / Appell-Lerch / Bailey
        "mock_theta_f3" | "mock_theta_phi3" | "mock_theta_psi3" | "mock_theta_chi3" |
        "mock_theta_omega3" | "mock_theta_nu3" | "mock_theta_rho3" |
        "mock_theta_f0_5" | "mock_theta_f1_5" | "mock_theta_cap_f0_5" | "mock_theta_cap_f1_5" |
        "mock_theta_phi0_5" | "mock_theta_phi1_5" | "mock_theta_psi0_5" | "mock_theta_psi1_5" |
        "mock_theta_chi0_5" | "mock_theta_chi1_5" |
        "mock_theta_cap_f0_7" | "mock_theta_cap_f1_7" | "mock_theta_cap_f2_7" => "(order)".to_string(),
        "appell_lerch_m" => "(a_pow, z_pow, order)".to_string(),
        "universal_mock_theta_g2" | "universal_mock_theta_g3" => "(a_pow, order)".to_string(),
        "bailey_weak_lemma" => "(pair_code, a_num, a_den, a_pow, max_n, order)".to_string(),
        "bailey_apply_lemma" => "(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, max_n, order)".to_string(),
        "bailey_chain" => "(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, depth, max_n, order)".to_string(),
        "bailey_discover" => "(lhs, rhs, a_num, a_den, a_pow, max_depth, order)".to_string(),
        // Group 8: Identity Proving
        "prove_eta_id" => "(terms_list, level)".to_string(),
        "search_identities" => "(search_type)".to_string(),
        "q_gosper" => "(upper_list, lower_list, z_num, z_den, z_pow, q_num, q_den)".to_string(),
        "q_zeilberger" => "(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order)".to_string(),
        "verify_wz" => "(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order, max_k)".to_string(),
        "q_petkovsek" => "(coeff_list, q_num, q_den)".to_string(),
        "prove_nonterminating" => "(requires Python API)".to_string(),
        // Group 9: Script loading
        "read" => "(filename)".to_string(),
        // Group 10: Variable management
        "anames" => "()".to_string(),
        "restart" => "()".to_string(),
        // Group 11: Jacobi Products
        "jac" | "JAC" => "(a, b) -- Jacobi product factor (q^a;q^b)_inf".to_string(),
        "theta" => "(z, q, T) -- general theta series sum(z^i * q^(i^2), i=-T..T)".to_string(),
        "jac2prod" => "(JP, q, T) -- convert Jacobi product to explicit product form".to_string(),
        "jac2series" => "(jacexpr, T) or (JP, q, T) -- convert Jacobi product to q-series".to_string(),
        "qs2jaccombo" => "(f, q, T) -- decompose q-series into sum of Jacobi products".to_string(),
        // Group Q: Expression operations
        "series" => "(expr, q, T)".to_string(),
        "expand" => "(expr) or (expr, q, T)".to_string(),
        // Group P: Number theory
        "floor" => "(x)".to_string(),
        "legendre" => "(m, p)".to_string(),
        "min" => "(a, b, ...) -- minimum of integer/rational values".to_string(),
        "max" => "(a, b, ...) -- maximum of integer/rational values".to_string(),
        // Group R: Polynomial operations
        "factor" => "(poly)".to_string(),
        // Group S: Substitution
        "subs" => "(var=val, ..., expr)".to_string(),
        // Group T: Simplification
        "radsimp" => "(expr) -- simplify rational series expression".to_string(),
        // Group U: List Operations
        "nops" => "(expr) -- number of operands/elements".to_string(),
        "op" => "(i, expr) -- extract i-th operand (1-indexed)".to_string(),
        "map" => "(f, list) -- apply function to each element".to_string(),
        "sort" => "(list) -- sort list elements".to_string(),
        // Group W: Iteration
        "add" => "(expr, i=a..b)".to_string(),
        "mul" => "(expr, i=a..b)".to_string(),
        "seq" => "(expr, i=a..b)".to_string(),
        // Group V: Series Coefficient & Utility
        "coeff" => "(f, q, n) -- coefficient of q^n in series f".to_string(),
        "degree" => "(f, q) -- highest degree of q in polynomial/series f".to_string(),
        "numer" => "(x) -- numerator of rational number".to_string(),
        "denom" => "(x) -- denominator of rational number".to_string(),
        "modp" => "(a, p) -- a mod p (non-negative)".to_string(),
        "mods" => "(a, p) -- a mod p (symmetric, centered at 0)".to_string(),
        "type" => "(expr, t) -- check if expr has type t".to_string(),
        "evalb" => "(expr) -- evaluate expression as boolean".to_string(),
        "cat" => "(s1, s2, ...) -- concatenate arguments into a name".to_string(),
        // Package info
        "changes" => "() -- print recent changes to q-Kangaroo".to_string(),
        "packageversion" => "() -- print package version".to_string(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Alias resolution
// ---------------------------------------------------------------------------

/// Resolve a Maple alias to its canonical q-Kangaroo name.
///
/// Case-insensitive lookup. If no alias matches, returns the input unchanged.
fn resolve_alias(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "partition_count" => "numbpart".to_string(),
        "rankgf" => "rank_gf".to_string(),
        "crankgf" => "crank_gf".to_string(),
        "qphihyper" => "phi".to_string(),
        "qpsihyper" => "psi".to_string(),
        "qgauss" => "try_summation".to_string(),
        "proveid" => "prove_eta_id".to_string(),
        "qzeil" => "q_zeilberger".to_string(),
        "qzeilberger" => "q_zeilberger".to_string(),
        "qpetkovsek" => "q_petkovsek".to_string(),
        "qgosper" => "q_gosper".to_string(),
        "findlincombo_modp" => "findlincombomodp".to_string(),
        "findhom_modp" => "findhommodp".to_string(),
        "findhomcombo_modp" => "findhomcombomodp".to_string(),
        "search_id" => "search_identities".to_string(),
        "g2" => "universal_mock_theta_g2".to_string(),
        "g3" => "universal_mock_theta_g3".to_string(),
        "l" => "legendre".to_string(),
        _ => name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Fuzzy matching for "Did you mean?" suggestions
// ---------------------------------------------------------------------------

/// All canonical function names (120 functions) for fuzzy matching.
/// (print is special-cased before dispatch and not included here)
const ALL_FUNCTION_NAMES: &[&str] = &[
    // Pattern A: Series generators
    "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
    "theta2", "theta3", "theta4",
    "partition_gf", "distinct_parts_gf", "odd_parts_gf", "bounded_parts_gf",
    "rank_gf", "crank_gf",
    // Pattern B: No-session
    "numbpart",
    // Pattern C: Series-input analysis
    "sift", "qdegree", "lqdegree", "lqdegree0", "qfactor", "zqfactor",
    "checkmult", "checkprod",
    "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
    // Pattern D: Target + candidates
    "findlincombo", "findhomcombo", "findnonhomcombo",
    "findlincombomodp", "findhomcombomodp",
    // Pattern E: List of series
    "findhom", "findnonhom", "findhommodp", "findmaxind", "findprod", "findcong",
    // Pattern F: Two series
    "findpoly",
    // Pattern G: Hypergeometric
    "phi", "psi", "try_summation", "heine1", "heine2", "heine3",
    "sears_transform", "watson_transform",
    // Pattern H: Identity proving
    "prove_eta_id", "search_identities",
    // Pattern I: Mock theta / Appell-Lerch
    "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3",
    "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
    "mock_theta_f0_5", "mock_theta_f1_5",
    "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
    "mock_theta_phi0_5", "mock_theta_phi1_5",
    "mock_theta_psi0_5", "mock_theta_psi1_5",
    "mock_theta_chi0_5", "mock_theta_chi1_5",
    "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
    "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
    // Pattern J: Bailey
    "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain", "bailey_discover",
    // Pattern K: Algorithmic
    "q_gosper", "q_zeilberger", "verify_wz", "q_petkovsek",
    // Pattern L: Nonterminating
    "prove_nonterminating", "find_transformation_chain",
    // Pattern M: Script loading
    "read",
    // Pattern N: Variable management
    "anames", "restart",
    // Pattern O: Jacobi Products
    "JAC", "theta", "jac2prod", "jac2series", "qs2jaccombo",
    // Pattern Q: Expression operations
    "series", "expand",
    // Pattern P: Number theory
    "floor", "legendre", "min", "max",
    // Pattern R: Polynomial operations
    "factor",
    // Pattern S: Substitution
    "subs",
    // Pattern T: Simplification
    "radsimp",
    // Pattern U: List operations
    "nops", "op", "map", "sort",
    // Pattern V: Series Coefficient & Utility
    "coeff", "degree", "numer", "denom", "modp", "mods", "type", "evalb", "cat",
    // Pattern W: Iteration
    "add", "mul", "seq",
    // Pattern X: Package info
    "changes", "packageversion",
];

/// All alias names for fuzzy matching.
const ALL_ALIAS_NAMES: &[&str] = &[
    "partition_count", "rankgf", "crankgf", "qphihyper", "qpsihyper",
    "qgauss", "proveid", "qzeil", "qzeilberger", "qpetkovsek",
    "qgosper", "findlincombo_modp", "findhom_modp", "findhomcombo_modp", "search_id",
    "g2", "g3", "L",
];

/// Find function names similar to `unknown` using edit distance.
///
/// Returns up to 3 suggestions sorted by edit distance.
fn find_similar_names(unknown: &str) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = Vec::new();
    let lower = unknown.to_lowercase();

    for &name in ALL_FUNCTION_NAMES.iter().chain(ALL_ALIAS_NAMES.iter()) {
        let dist = edit_distance(&lower, &name.to_lowercase());
        if dist <= 3 || name.contains(unknown) || unknown.contains(name) {
            scored.push((dist, name));
        }
    }

    scored.sort_by_key(|(d, _)| *d);
    scored.dedup_by(|a, b| a.1 == b.1);
    scored.into_iter().take(3).map(|(_, n)| n.to_string()).collect()
}

/// Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            dp[i + 1][j + 1] = (dp[i][j] + cost)
                .min(dp[i + 1][j] + 1)
                .min(dp[i][j + 1] + 1);
        }
    }
    dp[m][n]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::ast::{AstNode, BinOp, BoolBinOp, CompOp, Stmt, Terminator};
    use qsym_core::number::{QInt, QRat};

    fn make_env() -> Environment {
        Environment::new()
    }

    // --- Literal evaluation ---

    #[test]
    fn eval_integer_literal() {
        let mut env = make_env();
        let val = eval_expr(&AstNode::Integer(42), &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn eval_big_integer_literal() {
        let mut env = make_env();
        let val = eval_expr(
            &AstNode::BigInteger("99999999999999999999".to_string()),
            &mut env,
        )
        .unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(format!("{}", n), "99999999999999999999");
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn eval_q_returns_symbol() {
        let mut env = make_env();
        let val = eval_expr(&AstNode::Variable("q".to_string()), &mut env).unwrap();
        if let Value::Symbol(name) = val {
            assert_eq!(name, "q");
        } else {
            panic!("expected Symbol(\"q\"), got {:?}", val);
        }
    }

    #[test]
    fn eval_infinity() {
        let mut env = make_env();
        let val = eval_expr(&AstNode::Infinity, &mut env).unwrap();
        assert!(matches!(val, Value::Infinity));
    }

    // --- Variable lookup ---

    #[test]
    fn eval_variable_found() {
        let mut env = make_env();
        env.set_var("f", Value::Integer(QInt::from(7i64)));
        let val = eval_expr(&AstNode::Variable("f".to_string()), &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_variable_not_found_returns_symbol() {
        let mut env = make_env();
        let val = eval_expr(&AstNode::Variable("unknown".to_string()), &mut env).unwrap();
        if let Value::Symbol(name) = val {
            assert_eq!(name, "unknown");
        } else {
            panic!("expected Symbol(\"unknown\"), got {:?}", val);
        }
    }

    // --- Last result ---

    #[test]
    fn eval_last_result_found() {
        let mut env = make_env();
        env.last_result = Some(Value::Integer(QInt::from(99i64)));
        let val = eval_expr(&AstNode::LastResult, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(99i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_last_result_missing() {
        let mut env = make_env();
        let err = eval_expr(&AstNode::LastResult, &mut env).unwrap_err();
        assert!(matches!(err, EvalError::NoLastResult));
    }

    // --- Arithmetic ---

    #[test]
    fn eval_add_integers() {
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(4)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_mul_integers() {
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Mul,
            lhs: Box::new(AstNode::Integer(6)),
            rhs: Box::new(AstNode::Integer(7)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_sub_integers() {
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Sub,
            lhs: Box::new(AstNode::Integer(10)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_div_integers_gives_rational() {
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Div,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(4)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Rational(r) = val {
            assert_eq!(r, QRat::from((3i64, 4i64)));
        } else {
            panic!("expected Rational, got {:?}", val);
        }
    }

    #[test]
    fn eval_pow_integer() {
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Pow,
            lhs: Box::new(AstNode::Integer(2)),
            rhs: Box::new(AstNode::Integer(10)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(1024i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_neg_integer() {
        let mut env = make_env();
        let node = AstNode::Neg(Box::new(AstNode::Integer(5)));
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(-5i64));
        } else {
            panic!("expected Integer");
        }
    }

    #[test]
    fn eval_series_add() {
        let mut env = make_env();
        // Assign q-series values to variables, then add them
        let q_fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, 20);
        env.set_var("a", Value::Series(q_fps.clone()));
        env.set_var("b", Value::Series(q_fps));
        // a + b = 2*q
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Variable("a".to_string())),
            rhs: Box::new(AstNode::Variable("b".to_string())),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(1), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    #[test]
    fn eval_scalar_mul_series() {
        let mut env = make_env();
        // Assign q-series to variable, then multiply by scalar
        let q_fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, 20);
        env.set_var("s", Value::Series(q_fps));
        // 3 * s -> 3*q
        let node = AstNode::BinOp {
            op: BinOp::Mul,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Variable("s".to_string())),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(1), QRat::from((3i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    #[test]
    fn eval_series_plus_integer() {
        let mut env = make_env();
        // Assign q-series to variable, then add integer
        let q_fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, 20);
        env.set_var("s", Value::Series(q_fps));
        // s + 1 -> 1 + q + O(q^20)
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Variable("s".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    // --- Symbol Arithmetic ---

    #[test]
    fn eval_symbol_pow() {
        // q^2 -> Series with one term at power 2, coefficient 1
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Pow,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Integer(2)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(2), QRat::one());
            assert_eq!(fps.coeff(0), QRat::zero());
            assert_eq!(fps.coeff(1), QRat::zero());
            assert!(fps.truncation_order() >= POLYNOMIAL_ORDER);
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_pow_negative() {
        // q^(-1) -> Series with one term at power -1
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Pow,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Neg(Box::new(AstNode::Integer(1)))),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(-1), QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_mul_int() {
        // 2*q^3 -> Series with one term at power 3, coefficient 2
        let mut env = make_env();
        // Build: 2 * (q ^ 3)
        let node = AstNode::BinOp {
            op: BinOp::Mul,
            lhs: Box::new(AstNode::Integer(2)),
            rhs: Box::new(AstNode::BinOp {
                op: BinOp::Pow,
                lhs: Box::new(AstNode::Variable("q".to_string())),
                rhs: Box::new(AstNode::Integer(3)),
            }),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(3), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(0), QRat::zero());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_add() {
        // q + 1 -> Series with two terms (constant 1, power 1 coefficient 1)
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_polynomial_arithmetic() {
        // (q+1)*(q+1) -> 1 + 2*q + q^2
        let mut env = make_env();
        let q_plus_1 = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        let node = AstNode::BinOp {
            op: BinOp::Mul,
            lhs: Box::new(q_plus_1.clone()),
            rhs: Box::new(q_plus_1),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(2), QRat::one());
            assert!(fps.truncation_order() >= POLYNOMIAL_ORDER);
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_negate() {
        // -q -> Series with one term at power 1, coefficient -1
        let mut env = make_env();
        let node = AstNode::Neg(Box::new(AstNode::Variable("q".to_string())));
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(1), -QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_sub() {
        // q - 1 -> Series with constant -1 and q^1 coefficient 1
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Sub,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), -QRat::one());
            assert_eq!(fps.coeff(1), QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_symbol_div_scalar() {
        // q / 2 -> Series with one term at power 1, coefficient 1/2
        let mut env = make_env();
        let node = AstNode::BinOp {
            op: BinOp::Div,
            lhs: Box::new(AstNode::Variable("q".to_string())),
            rhs: Box::new(AstNode::Integer(2)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(1), QRat::from((1i64, 2i64)));
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    // --- Assignment ---

    #[test]
    fn eval_assignment_stores_variable() {
        let mut env = make_env();
        let node = AstNode::Assign {
            name: "x".to_string(),
            value: Box::new(AstNode::Integer(42)),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = &val {
            assert_eq!(*n, QInt::from(42i64));
        } else {
            panic!("expected Integer");
        }
        // Check it's in the environment
        let stored = env.get_var("x").unwrap();
        if let Value::Integer(n) = stored {
            assert_eq!(*n, QInt::from(42i64));
        } else {
            panic!("expected Integer in env");
        }
    }

    // --- Statement evaluation ---

    #[test]
    fn eval_stmt_semi_returns_some() {
        let mut env = make_env();
        let stmt = Stmt {
            node: AstNode::Integer(42),
            terminator: Terminator::Semi,
        };
        let result = eval_stmt(&stmt, &mut env).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn eval_stmt_colon_returns_none() {
        let mut env = make_env();
        let stmt = Stmt {
            node: AstNode::Integer(42),
            terminator: Terminator::Colon,
        };
        let result = eval_stmt(&stmt, &mut env).unwrap();
        assert!(result.is_none());
        // But last_result is still set
        assert!(env.last_result.is_some());
    }

    #[test]
    fn eval_stmt_sets_last_result() {
        let mut env = make_env();
        let stmt = Stmt {
            node: AstNode::Integer(99),
            terminator: Terminator::Semi,
        };
        eval_stmt(&stmt, &mut env).unwrap();
        if let Some(Value::Integer(n)) = &env.last_result {
            assert_eq!(*n, QInt::from(99i64));
        } else {
            panic!("expected last_result to be Integer(99)");
        }
    }

    // --- Levenshtein ---

    #[test]
    fn edit_distance_identical() {
        assert_eq!(edit_distance("etaq", "etaq"), 0);
    }

    #[test]
    fn edit_distance_one_delete() {
        assert_eq!(edit_distance("etaq", "eta"), 1);
    }

    #[test]
    fn edit_distance_one_insert() {
        assert_eq!(edit_distance("eta", "etaq"), 1);
    }

    #[test]
    fn edit_distance_substitution() {
        assert_eq!(edit_distance("etaq", "etax"), 1);
    }

    #[test]
    fn find_similar_names_close_match() {
        let suggestions = find_similar_names("etaq2");
        assert!(
            suggestions.contains(&"etaq".to_string()),
            "expected 'etaq' in {:?}",
            suggestions
        );
    }

    #[test]
    fn find_similar_names_exact_match() {
        let suggestions = find_similar_names("etaq");
        assert!(
            suggestions.contains(&"etaq".to_string()),
            "expected 'etaq' in {:?}",
            suggestions
        );
    }

    // --- Alias resolution ---

    #[test]
    fn resolve_alias_numbpart() {
        assert_eq!(resolve_alias("partition_count"), "numbpart");
    }

    #[test]
    fn resolve_alias_case_insensitive() {
        assert_eq!(resolve_alias("PARTITION_COUNT"), "numbpart");
        assert_eq!(resolve_alias("QZeil"), "q_zeilberger");
    }

    #[test]
    fn resolve_alias_passthrough() {
        assert_eq!(resolve_alias("aqprod"), "aqprod");
        assert_eq!(resolve_alias("etaq"), "etaq");
    }

    #[test]
    fn resolve_alias_all_maple_names() {
        assert_eq!(resolve_alias("partition_count"), "numbpart");
        assert_eq!(resolve_alias("rankgf"), "rank_gf");
        assert_eq!(resolve_alias("crankgf"), "crank_gf");
        assert_eq!(resolve_alias("qphihyper"), "phi");
        assert_eq!(resolve_alias("qpsihyper"), "psi");
        assert_eq!(resolve_alias("qgauss"), "try_summation");
        assert_eq!(resolve_alias("proveid"), "prove_eta_id");
        assert_eq!(resolve_alias("qzeilberger"), "q_zeilberger");
        assert_eq!(resolve_alias("qpetkovsek"), "q_petkovsek");
        assert_eq!(resolve_alias("qgosper"), "q_gosper");
        assert_eq!(resolve_alias("findlincombo_modp"), "findlincombomodp");
        assert_eq!(resolve_alias("findhom_modp"), "findhommodp");
        assert_eq!(resolve_alias("findhomcombo_modp"), "findhomcombomodp");
        assert_eq!(resolve_alias("search_id"), "search_identities");
    }

    // --- Panic catching ---

    #[test]
    fn eval_stmt_safe_catches_panic() {
        let mut env = make_env();
        // Trigger a panic: divide series with zero constant term
        // Create series q + O(q^20), then try to invert via 1/q
        // Actually, let's set up a variable holding 0 + O(q^20) and divide
        let zero_fps = FormalPowerSeries::zero(env.sym_q, 20);
        env.set_var("z", Value::Series(zero_fps));

        // 1 / z -- this will panic because invert requires nonzero constant term
        let stmt = Stmt {
            node: AstNode::BinOp {
                op: BinOp::Div,
                lhs: Box::new(AstNode::Integer(1)),
                rhs: Box::new(AstNode::Variable("z".to_string())),
            },
            terminator: Terminator::Semi,
        };
        let result = eval_stmt_safe(&stmt, &mut env);
        match result {
            Err(EvalError::Panic(msg)) => {
                assert!(
                    msg.contains("constant term is zero"),
                    "expected translated panic about constant term, got: {}",
                    msg
                );
            }
            other => panic!("expected EvalError::Panic, got {:?}", other),
        }
    }

    // --- List evaluation ---

    #[test]
    fn eval_list_literal() {
        let mut env = make_env();
        let node = AstNode::List(vec![
            AstNode::Integer(1),
            AstNode::Integer(2),
            AstNode::Integer(3),
        ]);
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::List(items) = val {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected List");
        }
    }

    // --- Type errors ---

    #[test]
    fn type_error_add_bool_integer() {
        let mut env = make_env();
        env.set_var("b", Value::Bool(true));
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Variable("b".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        let err = eval_expr(&node, &mut env).unwrap_err();
        assert!(matches!(err, EvalError::TypeError { .. }));
    }

    // --- Error display ---

    #[test]
    fn eval_error_display_unknown_var() {
        let err = EvalError::UnknownVariable { name: "xyz".to_string() };
        let msg = format!("{}", err);
        assert!(msg.contains("undefined variable 'xyz'"));
    }

    #[test]
    fn eval_error_display_unknown_func() {
        let err = EvalError::UnknownFunction {
            name: "etaq2".to_string(),
            suggestions: vec!["etaq".to_string()],
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unknown function 'etaq2'"));
        assert!(msg.contains("Did you mean: etaq?"));
    }

    #[test]
    fn eval_error_display_wrong_arg_count() {
        let err = EvalError::WrongArgCount {
            function: "aqprod".to_string(),
            expected: "4".to_string(),
            got: 2,
            signature: "a, q, n, N".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("aqprod expects 4 arguments"));
        assert!(msg.contains("got 2"));
    }

    #[test]
    fn eval_error_display_panic() {
        let err = EvalError::Panic("division by zero".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("computation failed: division by zero"));
    }

    // --- Value::type_name ---

    #[test]
    fn value_type_names() {
        assert_eq!(Value::Integer(QInt::from(1i64)).type_name(), "integer");
        assert_eq!(Value::Rational(QRat::from((1i64, 2i64))).type_name(), "rational");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::None.type_name(), "none");
        assert_eq!(Value::Infinity.type_name(), "infinity");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::Dict(vec![]).type_name(), "dict");
    }

    // --- Argument extraction helpers ---

    #[test]
    fn expect_args_correct_count() {
        let args = vec![Value::Integer(QInt::from(1i64)), Value::Integer(QInt::from(2i64))];
        assert!(expect_args("test", &args, 2).is_ok());
    }

    #[test]
    fn expect_args_wrong_count() {
        let args = vec![Value::Integer(QInt::from(1i64))];
        assert!(expect_args("test", &args, 2).is_err());
    }

    #[test]
    fn extract_i64_from_integer() {
        let args = vec![Value::Integer(QInt::from(42i64))];
        assert_eq!(extract_i64("test", &args, 0).unwrap(), 42);
    }

    #[test]
    fn extract_i64_from_non_integer() {
        let args = vec![Value::Bool(true)];
        assert!(extract_i64("test", &args, 0).is_err());
    }

    #[test]
    fn extract_qrat_from_integer() {
        let args = vec![Value::Integer(QInt::from(3i64))];
        let r = extract_qrat("test", &args, 0).unwrap();
        assert_eq!(r, QRat::from((3i64, 1i64)));
    }

    #[test]
    fn extract_qrat_from_rational() {
        let args = vec![Value::Rational(QRat::from((3i64, 7i64)))];
        let r = extract_qrat("test", &args, 0).unwrap();
        assert_eq!(r, QRat::from((3i64, 7i64)));
    }

    #[test]
    fn extract_bool_ok() {
        let args = vec![Value::Bool(true)];
        assert_eq!(extract_bool("test", &args, 0).unwrap(), true);
    }

    // --- Dispatch: Group 1 (q-Pochhammer and Products) ---

    #[test]
    fn dispatch_etaq_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("etaq", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // etaq(1,1,20) = (q;q)_inf = 1 - q - q^2 + q^5 + q^7 - ...
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), -QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_aqprod_with_infinity() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),  // coeff_num
            Value::Integer(QInt::from(1i64)),  // coeff_den
            Value::Integer(QInt::from(1i64)),  // power
            Value::Infinity,                    // n = infinity
            Value::Integer(QInt::from(10i64)), // order
        ];
        let val = dispatch("aqprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_aqprod_finite() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(3i64)),  // n = 3
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("aqprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_qbin_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("qbin", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_jacprod_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jacprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_tripleprod_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("tripleprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_quinprod_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    // --- Dispatch: Group 1 Maple-style ---

    /// Helper: create a monomial series q^power for test arguments.
    fn make_monomial_series(env: &Environment, power: i64) -> Value {
        let fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), power, POLYNOMIAL_ORDER);
        Value::Series(fps)
    }

    #[test]
    fn dispatch_jacprod_maple_style() {
        let mut env = make_env();
        // Maple: jacprod(1, 5, q, 30) = JAC(1,5) / JAC(5,15)
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(30i64)),
        ];
        let val = dispatch("jacprod", &args, &mut env).unwrap();
        if let Value::Series(ref fps) = val {
            assert_eq!(fps.coeff(0), QRat::one(), "constant term should be 1");
        } else {
            panic!("expected Series, got {:?}", val);
        }

        // Verify Maple result differs from legacy JAC(1,5) at some coefficient
        // (since Maple = JAC(a,b)/JAC(b,3b), legacy = JAC(a,b))
        let legacy_args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(30i64)),
        ];
        let legacy_val = dispatch("jacprod", &legacy_args, &mut env).unwrap();
        if let (Value::Series(maple_fps), Value::Series(legacy_fps)) = (&val, &legacy_val) {
            // Check several coefficients -- they must differ somewhere
            let mut found_diff = false;
            for k in 0..30 {
                if maple_fps.coeff(k) != legacy_fps.coeff(k) {
                    found_diff = true;
                    break;
                }
            }
            assert!(found_diff, "Maple and legacy jacprod should differ at some coefficient");
        }
    }

    #[test]
    fn dispatch_tripleprod_maple_style() {
        let mut env = make_env();
        // Maple: tripleprod(q^3, q, 20)
        let args = vec![
            make_monomial_series(&env, 3),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("tripleprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)), "expected Series");
    }

    #[test]
    fn dispatch_quinprod_maple_style() {
        let mut env = make_env();
        // Maple: quinprod(q^2, q, 20)
        let args = vec![
            make_monomial_series(&env, 2),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)), "expected Series");
    }

    #[test]
    fn dispatch_winquist_maple_style() {
        let mut env = make_env();
        // Maple: winquist(q, q^2, q, 10) -- a=q^1, b=q^2
        let args = vec![
            make_monomial_series(&env, 1),
            make_monomial_series(&env, 2),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)), "expected Series");
    }

    #[test]
    fn dispatch_qbin_garvan_3arg() {
        let mut env = make_env();
        // Garvan: qbin(q, 2, 4) -- exact polynomial [4 choose 2]_q
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(4i64)),
        ];
        let val = dispatch("qbin", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // [4 choose 2]_q = 1 + q + 2*q^2 + q^3 + q^4
            assert_eq!(fps.coeff(0), QRat::one(), "constant term");
            assert_eq!(fps.coeff(2), QRat::from(QInt::from(2i64)), "q^2 coefficient should be 2");
            assert_eq!(fps.coeff(4), QRat::one(), "q^4 coefficient should be 1");
            assert!(fps.truncation_order() >= POLYNOMIAL_ORDER, "should be exact polynomial");
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_qbin_4arg() {
        let mut env = make_env();
        // Extended: qbin(4, 2, q, 10) -- with explicit variable and truncation
        let args = vec![
            Value::Integer(QInt::from(4i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("qbin", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // Same result as [4 choose 2]_q but with explicit truncation
            assert_eq!(fps.coeff(0), QRat::one(), "constant term");
            assert_eq!(fps.coeff(2), QRat::from(QInt::from(2i64)), "q^2 coefficient");
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_etaq_multi_delta() {
        let mut env = make_env();
        // Multi-delta: etaq(q, [1, 2], 10)
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(2i64)),
            ]),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("etaq", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // etaq(q, [1, 2], 10) = etaq(q, 1, 10) * etaq(q, 2, 10)
            // = (q;q)_inf * (q^2;q^2)_inf
            assert_eq!(fps.coeff(0), QRat::one(), "constant term should be 1");
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_etaq_empty_list_errors() {
        let mut env = make_env();
        // etaq(q, [], 10) should error
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::List(vec![]),
            Value::Integer(QInt::from(10i64)),
        ];
        let result = dispatch("etaq", &args, &mut env);
        assert!(result.is_err(), "empty delta list should return error");
    }

    #[test]
    fn dispatch_etaq_negative_delta_errors() {
        let mut env = make_env();
        // etaq(q, [-1], 10) should error
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::List(vec![Value::Integer(QInt::from(-1i64))]),
            Value::Integer(QInt::from(10i64)),
        ];
        let result = dispatch("etaq", &args, &mut env);
        assert!(result.is_err(), "negative delta should return error");
    }

    // --- Dispatch: Group 2 (Partitions) ---

    #[test]
    fn dispatch_numbpart_5() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64))];
        let val = dispatch("numbpart", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_numbpart_100() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(100i64))];
        let val = dispatch("numbpart", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(190569292i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_numbpart_bounded() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64)), Value::Integer(QInt::from(3i64))];
        let val = dispatch("numbpart", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            // Partitions of 5 with max part 3: 3+2, 3+1+1, 2+2+1, 2+1+1+1, 1+1+1+1+1
            assert_eq!(n, QInt::from(5i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_partition_count_alias() {
        // partition_count is now an alias for numbpart -- should still work via resolve_alias
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64))];
        let resolved = resolve_alias("partition_count");
        let val = dispatch(&resolved, &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_partition_gf_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("partition_gf", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // p(0) = 1, p(1) = 1, p(2) = 2, p(3) = 3, p(4) = 5, p(5) = 7
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::one());
            assert_eq!(fps.coeff(5), QRat::from((7i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    #[test]
    fn dispatch_distinct_parts_gf_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("distinct_parts_gf", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_odd_parts_gf_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("odd_parts_gf", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_bounded_parts_gf_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("bounded_parts_gf", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_rank_gf_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("rank_gf", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_crank_gf_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("crank_gf", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    // --- Dispatch: Error handling ---

    #[test]
    fn dispatch_wrong_arg_count_etaq() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(1i64))];
        let err = dispatch("etaq", &args, &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("etaq expects 3 arguments"), "got: {}", msg);
        assert!(msg.contains("(b, t, order)"), "got: {}", msg);
    }

    #[test]
    fn dispatch_wrong_arg_type_etaq() {
        let mut env = make_env();
        let args = vec![
            Value::Bool(true),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let err = dispatch("etaq", &args, &mut env).unwrap_err();
        assert!(matches!(err, EvalError::ArgType { .. }));
    }

    #[test]
    fn dispatch_wrong_arg_count_aqprod() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(1i64))];
        let err = dispatch("aqprod", &args, &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("aqprod expects 5 arguments"), "got: {}", msg);
        assert!(msg.contains("(coeff_num, coeff_den, power, n_or_infinity, order)"), "got: {}", msg);
    }

    // --- Dispatch: Alias resolution ---

    #[test]
    fn dispatch_numbpart_alias() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64))];
        let val = dispatch("numbpart", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer");
        }
    }

    // --- Dispatch: Group 3 (Theta Functions) ---

    #[test]
    fn dispatch_theta2_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("theta2", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_theta3_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("theta3", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // theta3 = 1 + 2q + 2q^4 + 2q^9 + 2q^16 + ...
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(4), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(9), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    #[test]
    fn dispatch_theta4_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("theta4", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // theta4 = 1 - 2q + 2q^4 - 2q^9 + 2q^16 - ...
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((-2i64, 1i64)));
            assert_eq!(fps.coeff(4), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    // --- Dispatch: Group 4 (Series Analysis) ---

    #[test]
    fn dispatch_sift_returns_series() {
        let mut env = make_env();
        // Maple: sift(s, q, n, k, T)
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(30i64))], &mut env).unwrap();
        let sift_args = vec![
            pgf,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),  // n
            Value::Integer(QInt::from(0i64)),  // k
            Value::Integer(QInt::from(30i64)), // T
        ];
        let val = dispatch("sift", &sift_args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_sift_invalid_residue_errors() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(30i64))], &mut env).unwrap();
        // k=7 >= n=5 is invalid
        let sift_args = vec![
            pgf,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),  // n
            Value::Integer(QInt::from(7i64)),  // k (invalid: k >= n)
            Value::Integer(QInt::from(30i64)), // T
        ];
        let err = dispatch("sift", &sift_args, &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("residue"), "expected 'residue' in error message, got: {}", msg);
    }

    #[test]
    fn dispatch_qdegree_returns_integer() {
        let mut env = make_env();
        // qbin(5,2,20) is a polynomial of degree 6
        let qb = dispatch("qbin", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("qdegree", &[qb], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(6i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_lqdegree_returns_integer() {
        let mut env = make_env();
        // etaq(1,1,20) starts at q^0 (constant term 1)
        let eta = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("lqdegree", &[eta], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(0i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_qdegree_zero_series_returns_none() {
        let mut env = make_env();
        let zero = Value::Series(FormalPowerSeries::zero(env.sym_q, 20));
        let val = dispatch("qdegree", &[zero], &mut env).unwrap();
        assert!(matches!(val, Value::None));
    }

    #[test]
    fn dispatch_prodmake_returns_dict() {
        let mut env = make_env();
        // Maple: prodmake(f, q, T)
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("prodmake", &[
            pgf,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"exponents"), "expected 'exponents' in {:?}", keys);
            assert!(keys.contains(&"terms_used"), "expected 'terms_used' in {:?}", keys);
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_etamake_returns_eta_quotient() {
        let mut env = make_env();
        // Maple: etamake(f, q, T)
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("etamake", &[
            pgf,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ], &mut env).unwrap();
        if let Value::EtaQuotient { factors, q_shift } = &val {
            assert!(!factors.is_empty(), "expected non-empty factors");
            // partition_gf is 1/(q;q)_inf so etamake should give eta(tau)^{-1}
            assert_eq!(factors.get(&1), Some(&-1), "expected factor 1 -> -1");
        } else {
            panic!("expected EtaQuotient, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jacprodmake_returns_dict() {
        let mut env = make_env();
        // Maple: jacprodmake(f, q, T) -- 3-arg form
        let jp = dispatch("jacprod", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("jacprodmake", &[
            jp,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"factors"));
            assert!(keys.contains(&"is_exact"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jacprodmake_4arg_with_period() {
        let mut env = make_env();
        // Maple: jacprodmake(f, q, T, P) -- 4-arg form with period filter
        let jp = dispatch("jacprod", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("jacprodmake", &[
            jp,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
            Value::Integer(QInt::from(10i64)), // P = 10 (5 divides 10)
        ], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"factors"));
            assert!(keys.contains(&"is_exact"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_mprodmake_returns_dict() {
        let mut env = make_env();
        // Maple: mprodmake(f, q, T)
        let dp = dispatch("distinct_parts_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("mprodmake", &[
            dp,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ], &mut env).unwrap();
        assert!(matches!(val, Value::Dict(_)));
    }

    #[test]
    fn dispatch_qetamake_returns_dict() {
        let mut env = make_env();
        // Maple: qetamake(f, q, T)
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("qetamake", &[
            pgf,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"factors"));
            assert!(keys.contains(&"q_shift"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_qfactor_returns_qproduct() {
        let mut env = make_env();
        // Maple: qfactor(f, q) -- qbin(5,2,20) is a polynomial
        let qb = dispatch("qbin", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("qfactor", &[
            qb,
            Value::Symbol("q".to_string()),
        ], &mut env).unwrap();
        assert!(matches!(val, Value::QProduct { .. }), "expected QProduct, got {:?}", val);
    }

    // --- Integration tests (parse -> eval -> format) ---

    #[test]
    fn integration_etaq_end_to_end() {
        use crate::parser::parse;
        use crate::format::format_value;

        let mut env = make_env();
        let stmts = parse("etaq(1,1,20)").unwrap();
        assert_eq!(stmts.len(), 1);
        let result = eval_stmt(&stmts[0], &mut env).unwrap();
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(matches!(val, Value::Series(_)));
        let text = format_value(&val, &env.symbols);
        // (q;q)_inf = 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + ...
        assert!(text.contains("q"), "expected 'q' in: {}", text);
        assert!(text.contains("1"), "expected '1' in: {}", text);
    }

    #[test]
    fn integration_partition_count_end_to_end() {
        use crate::parser::parse;
        use crate::format::format_value;

        let mut env = make_env();
        let stmts = parse("partition_count(50)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "204226");
    }

    #[test]
    fn integration_variable_persistence() {
        use crate::parser::parse;

        let mut env = make_env();

        // f := etaq(1,1,20)
        let stmts = parse("f := etaq(1,1,20)").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();
        assert!(env.get_var("f").is_some());
        assert!(matches!(env.get_var("f").unwrap(), Value::Series(_)));

        // Maple: prodmake(f, q, 10)
        let stmts2 = parse("prodmake(f, q, 10)").unwrap();
        let result = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();
        assert!(matches!(result, Value::Dict(_)));
    }

    #[test]
    fn integration_theta3_end_to_end() {
        use crate::parser::parse;
        use crate::format::format_value;

        let mut env = make_env();
        let stmts = parse("theta3(20)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        // theta3 = 1 + 2*q + 2*q^4 + ...
        assert!(text.contains("1"), "expected '1' in: {}", text);
        assert!(text.contains("q"), "expected 'q' in: {}", text);
    }

    #[test]
    fn integration_sift_partition_congruence() {
        use crate::parser::parse;

        let mut env = make_env();

        // f := partition_gf(30)
        let stmts = parse("f := partition_gf(30)").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();

        // g := sift(f, q, 5, 4, 30)
        let stmts2 = parse("g := sift(f, q, 5, 4, 30)").unwrap();
        let result = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();

        // p(5n+4) should be divisible by 5 -- check first few coefficients
        if let Value::Series(fps) = result {
            for i in 0..fps.truncation_order() {
                let c = fps.coeff(i);
                if !c.is_zero() {
                    // c should be divisible by 5
                    let n = c.0.numer().clone();
                    let d = c.0.denom().clone();
                    assert_eq!(d, rug::Integer::from(1), "coefficient at {} not integer", i);
                    assert_eq!(
                        n.clone() % rug::Integer::from(5),
                        rug::Integer::from(0),
                        "p(5*{}+4) = {} not divisible by 5",
                        i,
                        n
                    );
                }
            }
        } else {
            panic!("expected Series");
        }
    }

    #[test]
    fn integration_qfactor_qbin() {
        use crate::parser::parse;

        let mut env = make_env();

        // Maple: qfactor(f, q) returns a QProduct
        let stmts = parse("qfactor(qbin(5, 2, 20), q)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        assert!(matches!(result, Value::QProduct { .. }), "expected QProduct, got {:?}", result);
    }

    #[test]
    fn integration_qfactor_displays_product_form() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("qfactor(aqprod(q, q, 5), q)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert!(text.contains("(1-q)"), "expected (1-q) in: {}", text);
        assert!(text.contains("(1-q^5)"), "expected (1-q^5) in: {}", text);
        assert!(!text.contains("scalar"), "should not show raw dict: {}", text);
    }

    #[test]
    fn integration_multi_statement() {
        use crate::parser::parse;

        let mut env = make_env();

        // "f := etaq(1,1,20); qdegree(f)"
        let stmts = parse("f := etaq(1,1,20); qdegree(f)").unwrap();
        assert_eq!(stmts.len(), 2);

        eval_stmt(&stmts[0], &mut env).unwrap();
        let result = eval_stmt(&stmts[1], &mut env).unwrap().unwrap();
        // etaq(1,1,20) has terms up to q^19, so qdegree should be 19 or close
        assert!(matches!(result, Value::Integer(_)));
    }

    #[test]
    fn integration_percent_reference() {
        use crate::parser::parse;

        let mut env = make_env();

        // Compute etaq then reference with %
        let stmts = parse("etaq(1,1,20)").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();

        let stmts2 = parse("qdegree(%)").unwrap();
        let result = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();
        assert!(matches!(result, Value::Integer(_)));
    }

    // --- Dispatch: Group 5 (Relation Discovery) ---

    #[test]
    fn dispatch_findlincombo_maple_style() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let etq = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let candidates = Value::List(vec![pgf.clone(), etq.clone()]);
        let sl = Value::List(vec![
            Value::Symbol("F1".to_string()),
            Value::Symbol("F2".to_string()),
        ]);
        let args = vec![
            pgf,
            candidates,
            sl,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("findlincombo", &args, &mut env).unwrap();
        // Should find a combination (first basis is identical to target)
        match val {
            Value::String(s) => {
                assert!(s.contains("F1"), "expected F1 label in output: {}", s);
            }
            Value::None => {} // also acceptable depending on truncation
            other => panic!("expected String or None, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_findlincombo_duplicate_sl_errors() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let candidates = Value::List(vec![pgf.clone(), pgf.clone()]);
        let sl = Value::List(vec![
            Value::Symbol("F1".to_string()),
            Value::Symbol("F1".to_string()), // duplicate
        ]);
        let args = vec![
            pgf,
            candidates,
            sl,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(0i64)),
        ];
        let result = dispatch("findlincombo", &args, &mut env);
        assert!(result.is_err(), "expected error for duplicate SL labels");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("duplicate label"), "error should mention duplicate: {}", err_msg);
    }

    #[test]
    fn dispatch_findlincombomodp_non_prime_errors() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let candidates = Value::List(vec![pgf.clone()]);
        let sl = Value::List(vec![Value::Symbol("F1".to_string())]);
        let args = vec![
            pgf,
            candidates,
            sl,
            Value::Integer(QInt::from(4i64)), // not prime
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(0i64)),
        ];
        let result = dispatch("findlincombomodp", &args, &mut env);
        assert!(result.is_err(), "expected error for non-prime p");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("not prime"), "error should mention not prime: {}", err_msg);
    }

    #[test]
    fn dispatch_findhom_maple_style() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let series_list = Value::List(vec![e1]);
        let args = vec![
            series_list,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(2i64)),  // degree
            Value::Integer(QInt::from(5i64)),  // topshift
        ];
        let val = dispatch("findhom", &args, &mut env).unwrap();
        assert!(matches!(val, Value::List(_)));
    }

    #[test]
    fn dispatch_findcong_garvan_style() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(100i64))], &mut env).unwrap();
        // findcong(pgf, 99) -- auto-scan moduli 2..floor(sqrt(99))
        let val = dispatch("findcong", &[pgf, Value::Integer(QInt::from(99i64))], &mut env).unwrap();
        if let Value::List(congruences) = val {
            assert!(!congruences.is_empty(), "expected at least one congruence");
            // Each entry should be a [B, A, R] triple
            let has_5n4 = congruences.iter().any(|c| {
                if let Value::List(items) = c {
                    items.len() == 3
                        && matches!(&items[0], Value::Integer(n) if n.0 == 4)
                        && matches!(&items[1], Value::Integer(n) if n.0 == 5)
                        && matches!(&items[2], Value::Integer(n) if n.0 == 5)
                } else {
                    false
                }
            });
            assert!(has_5n4, "Should find Ramanujan's p(5n+4) = 0 mod 5");
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findcong_with_lm() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(100i64))], &mut env).unwrap();
        // findcong(pgf, 99, 5) -- restrict to moduli 2..5
        let args = vec![
            pgf,
            Value::Integer(QInt::from(99i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("findcong", &args, &mut env).unwrap();
        if let Value::List(congruences) = val {
            // Should find p(5n+4) mod 5
            let has_5n4 = congruences.iter().any(|c| {
                if let Value::List(items) = c {
                    items.len() == 3
                        && matches!(&items[0], Value::Integer(n) if n.0 == 4)
                        && matches!(&items[1], Value::Integer(n) if n.0 == 5)
                } else {
                    false
                }
            });
            assert!(has_5n4, "Should find p(5n+4) with lm=5");
            // Should NOT find mod 7 results
            let has_mod7 = congruences.iter().any(|c| {
                if let Value::List(items) = c {
                    items.len() == 3
                        && matches!(&items[1], Value::Integer(n) if n.0 == 7)
                } else {
                    false
                }
            });
            assert!(!has_mod7, "Should not find mod 7 with lm=5");
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findmaxind_returns_list() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let e2 = dispatch("etaq", &[
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let list = Value::List(vec![e1, e2]);
        let args = vec![list, Value::Integer(QInt::from(5i64))];
        let val = dispatch("findmaxind", &args, &mut env).unwrap();
        if let Value::List(indices) = val {
            assert!(!indices.is_empty());
            // Should be 1-based indices
            for idx in &indices {
                if let Value::Integer(n) = idx {
                    assert!(n.0 >= 1, "indices should be 1-based");
                }
            }
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findpoly_maple_style() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let e2 = dispatch("etaq", &[
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let args = vec![
            e1, e2,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(2i64)),
        ];
        let val = dispatch("findpoly", &args, &mut env).unwrap();
        // Could be String (found relation) or None (no relation in that degree)
        match &val {
            Value::String(_) | Value::None => {}
            other => panic!("expected String or None, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_findhomcombo_maple_style() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let etq = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let candidates = Value::List(vec![pgf.clone(), etq.clone()]);
        // findhomcombo(f, L, q, n, topshift) -- 5 args
        let args = vec![
            pgf,
            candidates,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(1i64)), // degree 1
            Value::Integer(QInt::from(5i64)), // topshift
        ];
        let val = dispatch("findhomcombo", &args, &mut env).unwrap();
        match val {
            Value::String(s) => {
                // Should contain X[1] or X[2] labels
                assert!(s.contains("X[") || s == "0", "expected X[i] labels: {}", s);
            }
            Value::None => {} // no combination found
            other => panic!("expected String or None, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_findhommodp_p_before_q() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let series_list = Value::List(vec![e1]);
        // findhommodp(L, p, q, n, topshift) -- 5 args, p before q
        let args = vec![
            series_list,
            Value::Integer(QInt::from(5i64)),  // p
            Value::Symbol("q".to_string()),     // q
            Value::Integer(QInt::from(2i64)),  // degree
            Value::Integer(QInt::from(5i64)),  // topshift
        ];
        let val = dispatch("findhommodp", &args, &mut env).unwrap();
        assert!(matches!(val, Value::List(_)));
    }

    // --- Dispatch: Group 6 (Hypergeometric) ---

    #[test]
    fn dispatch_phi_returns_series() {
        let mut env = make_env();
        // phi([[1,1,0]], [], 1, 1, 1, 20) = 1_phi_0(1; ; q) = 1/(1-q)^1 up to truncation
        let upper = Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(0i64)),
            ]),
        ]);
        let lower = Value::List(vec![]);
        let args = vec![
            upper, lower,
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("phi", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_try_summation_returns_series_or_none() {
        let mut env = make_env();
        // Simple phi that may or may not match a summation formula
        let upper = Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(0i64)),
            ]),
        ]);
        let lower = Value::List(vec![]);
        let args = vec![
            upper, lower,
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("try_summation", &args, &mut env).unwrap();
        match val {
            Value::Series(_) | Value::None => {}
            other => panic!("expected Series or None, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_phi_wrong_args() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(1i64))];
        let err = dispatch("phi", &args, &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("phi expects 6 arguments"), "got: {}", msg);
    }

    // --- Dispatch: Group 7 (Mock Theta / Appell-Lerch / Bailey) ---

    #[test]
    fn dispatch_mock_theta_f3_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("mock_theta_f3", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // f3 starts with 1 + q + ...
            assert!(!fps.is_zero());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_mock_theta_phi3_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("mock_theta_phi3", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_mock_theta_rho3_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("mock_theta_rho3", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_mock_theta_f0_5_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("mock_theta_f0_5", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_mock_theta_cap_f0_7_returns_series() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("mock_theta_cap_f0_7", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_all_20_mock_theta_functions() {
        let mut env = make_env();
        let mock_theta_names = [
            "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3", "mock_theta_chi3",
            "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
            "mock_theta_f0_5", "mock_theta_f1_5",
            "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
            "mock_theta_phi0_5", "mock_theta_phi1_5",
            "mock_theta_psi0_5", "mock_theta_psi1_5",
            "mock_theta_chi0_5", "mock_theta_chi1_5",
            "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
        ];
        for &fname in &mock_theta_names {
            let args = vec![Value::Integer(QInt::from(15i64))];
            let val = dispatch(fname, &args, &mut env);
            assert!(
                val.is_ok(),
                "mock theta function {} failed: {:?}",
                fname,
                val.unwrap_err()
            );
            assert!(
                matches!(val.unwrap(), Value::Series(_)),
                "mock theta function {} did not return Series",
                fname
            );
        }
    }

    #[test]
    fn dispatch_appell_lerch_m_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(2i64)),   // a_pow
            Value::Integer(QInt::from(3i64)),   // z_pow
            Value::Integer(QInt::from(20i64)),  // order
        ];
        let val = dispatch("appell_lerch_m", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_g2_alias_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(3i64)),   // a_pow
            Value::Integer(QInt::from(20i64)),  // order
        ];
        let val = dispatch("g2", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_g3_alias_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(3i64)),   // a_pow
            Value::Integer(QInt::from(20i64)),  // order
        ];
        let val = dispatch("g3", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
    }

    #[test]
    fn dispatch_bailey_weak_lemma_returns_pair() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(0i64)),   // pair_code: 0=Unit
            Value::Integer(QInt::from(1i64)),   // a_num
            Value::Integer(QInt::from(1i64)),   // a_den
            Value::Integer(QInt::from(1i64)),   // a_pow (a = q)
            Value::Integer(QInt::from(3i64)),   // max_n
            Value::Integer(QInt::from(20i64)),  // order
        ];
        let val = dispatch("bailey_weak_lemma", &args, &mut env).unwrap();
        if let Value::Pair(lhs, rhs) = val {
            assert!(matches!(*lhs, Value::Series(_)));
            assert!(matches!(*rhs, Value::Series(_)));
        } else {
            panic!("expected Pair, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_bailey_discover_returns_dict() {
        let mut env = make_env();
        // Create two identical series, discover should find trivial match
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let e2 = e1.clone();
        let args = vec![
            e1, e2,
            Value::Integer(QInt::from(1i64)),   // a_num
            Value::Integer(QInt::from(1i64)),   // a_den
            Value::Integer(QInt::from(0i64)),   // a_pow (a = 1)
            Value::Integer(QInt::from(2i64)),   // max_depth
            Value::Integer(QInt::from(20i64)),  // order
        ];
        let val = dispatch("bailey_discover", &args, &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"found"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    // --- Dispatch: Group 8 (Identity Proving) ---

    #[test]
    fn dispatch_search_identities_returns_list() {
        let mut env = make_env();
        // search_type=0 returns all entries
        let args = vec![Value::Integer(QInt::from(0i64))];
        let val = dispatch("search_identities", &args, &mut env).unwrap();
        assert!(matches!(val, Value::List(_)));
    }

    #[test]
    fn dispatch_prove_nonterminating_returns_error() {
        let mut env = make_env();
        let err = dispatch("prove_nonterminating", &[], &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("Python API"), "got: {}", msg);
    }

    #[test]
    fn dispatch_q_gosper_returns_dict() {
        let mut env = make_env();
        // Simple 1phi0: summand is (a;q)_k * z^k / (q;q)_k
        let upper = Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(-2i64)),  // a = q^{-2} (terminates)
            ]),
        ]);
        let lower = Value::List(vec![]);
        let args = vec![
            upper, lower,
            Value::Integer(QInt::from(1i64)),   // z_num
            Value::Integer(QInt::from(1i64)),   // z_den
            Value::Integer(QInt::from(1i64)),   // z_pow
            Value::Integer(QInt::from(1i64)),   // q_num
            Value::Integer(QInt::from(2i64)),   // q_den (q=1/2)
        ];
        let val = dispatch("q_gosper", &args, &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"summable"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_q_petkovsek_returns_list() {
        let mut env = make_env();
        // Simple recurrence: S(n+1) = (1/2) * S(n) -> coefficients [1, -1/2]
        let coeffs = Value::List(vec![
            Value::Rational(QRat::from((1i64, 1i64))),
            Value::Rational(QRat::from((-1i64, 2i64))),
        ]);
        let args = vec![
            coeffs,
            Value::Integer(QInt::from(1i64)),   // q_num
            Value::Integer(QInt::from(2i64)),   // q_den
        ];
        let val = dispatch("q_petkovsek", &args, &mut env).unwrap();
        assert!(matches!(val, Value::List(_)));
    }

    // --- Comprehensive integration tests ---

    #[test]
    fn integration_mock_theta_f3() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("mock_theta_f3(20)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        assert!(matches!(result, Value::Series(_)));
    }

    #[test]
    fn integration_unknown_function_with_suggestions() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("etaq2(20)").unwrap();
        let err = eval_stmt(&stmts[0], &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("unknown function"), "got: {}", msg);
        assert!(msg.contains("etaq"), "expected suggestion 'etaq' in: {}", msg);
    }

    #[test]
    fn integration_wrong_arg_count_error() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("etaq(1)").unwrap();
        let err = eval_stmt(&stmts[0], &mut env).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("etaq expects 3 arguments"), "got: {}", msg);
    }

    #[test]
    fn integration_maple_alias_numbpart() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("numbpart(50)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "204226");
    }

    #[test]
    fn integration_list_syntax() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("[1, 2, 3]").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn integration_series_arithmetic() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("f := etaq(1,1,20)").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();
        let stmts2 = parse("g := etaq(2,1,20)").unwrap();
        eval_stmt(&stmts2[0], &mut env).unwrap();
        let stmts3 = parse("f * g").unwrap();
        let result = eval_stmt(&stmts3[0], &mut env).unwrap().unwrap();
        assert!(matches!(result, Value::Series(_)));
    }

    #[test]
    fn integration_format_etaq_descending_order() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("etaq(1,1,20)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        // Descending order: highest power first, constant "1" near end
        assert!(text.contains("+ 1 + O(q^20)"), "expected '+ 1 + O(q^20)' in descending output, got: {}", text);
        assert!(text.contains("q"), "expected 'q' in: {}", text);
    }

    #[test]
    fn integration_percent_42() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("42").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();
        let stmts2 = parse("%").unwrap();
        let result = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "42");
    }

    /// Verify that the ALL_FUNCTION_NAMES array has the expected count.
    #[test]
    fn function_count_verification() {
        // Count unique function names
        let count = ALL_FUNCTION_NAMES.len();
        // 7 (Group 1) + 7 (Group 2) + 3 (Group 3) + 9 (Group 4)
        // + 12 (Group 5) + 8+2 (Group 6: 6 heine/phi/psi + sears + watson + find_chain)
        // + 23 (Group 7: 20 mock + 3 appell/g)
        // + 4 (Bailey)
        // + 4 (q_gosper, q_zeil, verify_wz, q_petkovsek)
        // + 2 (prove_nonterminating, find_transformation_chain)
        // + 2 (prove_eta_id, search_identities)
        // = should be near 79
        assert!(
            count >= 85,
            "expected at least 85 function names in ALL_FUNCTION_NAMES, got {}",
            count
        );
    }

    // --- Panic message translation ---

    #[test]
    fn translate_panic_zero_constant_term() {
        let translated = translate_panic_message(
            "Cannot invert series with zero constant term",
        );
        assert!(translated.contains("cannot invert a series"));
        assert!(translated.contains("constant term is zero"));
    }

    #[test]
    fn translate_panic_division_by_zero() {
        let translated = translate_panic_message("QRat division by zero");
        assert_eq!(translated, "division by zero");
    }

    #[test]
    fn translate_panic_division_by_zero_uppercase() {
        let translated = translate_panic_message("Division by zero in rational");
        assert_eq!(translated, "division by zero");
    }

    #[test]
    fn translate_panic_cannot_invert_zero() {
        let translated = translate_panic_message("Cannot invert zero");
        assert_eq!(translated, "cannot invert zero");
    }

    #[test]
    fn translate_panic_index_out_of_bounds() {
        let translated = translate_panic_message("index out of bounds: the len is 5 but the index is 10");
        assert_eq!(translated, "index out of bounds");
    }

    #[test]
    fn translate_panic_unknown_passes_through() {
        let translated = translate_panic_message("some unknown error");
        assert_eq!(translated, "some unknown error");
    }

    #[test]
    fn translate_panic_thread_prefix_stripped() {
        let translated = translate_panic_message("thread 'main' panicked at 'attempt to divide by zero'");
        // Should strip the thread prefix
        assert!(!translated.contains("thread 'main'"));
    }

    // --- Phase 33 Plan 03: Symbol-aware function dispatch ---

    #[test]
    fn eval_etaq_with_symbol() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("etaq", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // etaq(q, 1, 5) = (q;q)_inf truncated at O(q^5) = 1 - q - q^2 + ...
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), -QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_etaq_with_custom_symbol() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("t".to_string()),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("etaq", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // Should use the "t" variable
            let sym_name = env.symbols.name(fps.variable());
            assert_eq!(sym_name, "t", "expected variable t, got {}", sym_name);
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn eval_aqprod_with_symbol_monomial() {
        let mut env = make_env();
        // aqprod(q, q, 3) -- q as monomial, q as var, n=3
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(3i64)),
        ];
        let val = dispatch("aqprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)), "expected Series");
    }

    #[test]
    fn eval_anames_empty() {
        let mut env = make_env();
        let val = dispatch("anames", &[], &mut env).unwrap();
        if let Value::List(items) = val {
            assert!(items.is_empty(), "expected empty list");
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn eval_anames_with_vars() {
        let mut env = make_env();
        env.set_var("x", Value::Integer(QInt::from(42i64)));
        env.set_var("y", Value::Integer(QInt::from(7i64)));
        let val = dispatch("anames", &[], &mut env).unwrap();
        if let Value::List(items) = val {
            let names: Vec<String> = items.iter().map(|v| {
                if let Value::String(s) = v { s.clone() } else { panic!("expected string") }
            }).collect();
            assert_eq!(names, vec!["x".to_string(), "y".to_string()]);
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn eval_restart_clears_vars() {
        let mut env = make_env();
        env.set_var("x", Value::Integer(QInt::from(42i64)));
        env.set_var("y", Value::Integer(QInt::from(7i64)));
        let val = dispatch("restart", &[], &mut env).unwrap();
        assert!(matches!(val, Value::String(ref s) if s == "Restart."));
        assert!(env.variables.is_empty(), "vars should be cleared after restart");
    }

    #[test]
    fn eval_unassign() {
        let mut env = make_env();
        // Set x := 42
        env.set_var("x", Value::Integer(QInt::from(42i64)));
        assert!(env.get_var("x").is_some());

        // Eval x := 'x' (Assign with StringLit)
        let node = AstNode::Assign {
            name: "x".to_string(),
            value: Box::new(AstNode::StringLit("x".to_string())),
        };
        let result = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(result, Value::Symbol(ref s) if s == "x"));
        assert!(env.get_var("x").is_none(), "x should be unassigned");

        // Now evaluating x should return Symbol
        let val = eval_expr(&AstNode::Variable("x".to_string()), &mut env).unwrap();
        assert!(matches!(val, Value::Symbol(ref s) if s == "x"));
    }

    // --- Dispatch: JacobiProduct (Task 1) ---

    #[test]
    fn dispatch_jac_returns_jacobi_product() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("JAC", &args, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = val {
            assert_eq!(factors, vec![(1, 5, 1)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac_lowercase() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("jac", &args, &mut env).unwrap();
        assert!(matches!(val, Value::JacobiProduct(_)));
    }

    #[test]
    fn dispatch_jac_negative_b_errors() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(-3i64)),
        ];
        let result = dispatch("JAC", &args, &mut env);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("positive integer"), "expected positive integer error, got: {}", msg);
    }

    #[test]
    fn dispatch_jac_zero_b_errors() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(0i64)),
        ];
        let result = dispatch("JAC", &args, &mut env);
        assert!(result.is_err());
    }

    #[test]
    fn eval_mul_jacobi_products() {
        let mut env = make_env();
        let left = Value::JacobiProduct(vec![(1, 5, 1)]);
        let right = Value::JacobiProduct(vec![(2, 5, 1)]);
        let result = eval_mul(left, right, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors.len(), 2);
            assert_eq!(factors, vec![(1, 5, 1), (2, 5, 1)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    #[test]
    fn eval_div_jacobi_products() {
        let mut env = make_env();
        let left = Value::JacobiProduct(vec![(1, 5, 1)]);
        let right = Value::JacobiProduct(vec![(2, 5, 1)]);
        let result = eval_div(left, right, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors.len(), 2);
            // (1,5,1) then (2,5,-1)
            assert_eq!(factors, vec![(1, 5, 1), (2, 5, -1)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_jacobi_product() {
        let mut env = make_env();
        let base = Value::JacobiProduct(vec![(1, 5, 1)]);
        let exp = Value::Integer(QInt::from(3i64));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors, vec![(1, 5, 3)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_jacobi_product_negative() {
        let mut env = make_env();
        let base = Value::JacobiProduct(vec![(1, 5, 1)]);
        let exp = Value::Integer(QInt::from(-2i64));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors, vec![(1, 5, -2)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    // --- eval_pow Rational exponent tests ---

    #[test]
    fn eval_pow_symbol_rational_integer_exponent() {
        // q^(4/2) = q^2 (Rational with denom=1 after simplification)
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((4i64, 2i64))); // 4/2 = 2/1
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(2), QRat::one());
            assert_eq!(fps.coeff(0), QRat::zero());
            assert_eq!(fps.coeff(1), QRat::zero());
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_symbol_rational_non_integer_fractional() {
        // q^(3/2) should produce FractionalPowerSeries
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((3i64, 2i64)));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 2);
            assert_eq!(inner.coeff(3), QRat::one());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_series_rational() {
        // (series)^(6/3) = (series)^2
        let mut env = make_env();
        let q_fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, 20);
        let base = Value::Series(q_fps);
        let exp = Value::Rational(QRat::from((6i64, 3i64))); // 6/3 = 2/1
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::Series(fps) = result {
            // q^2
            assert_eq!(fps.coeff(2), QRat::one());
            assert_eq!(fps.coeff(1), QRat::zero());
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_integer_rational() {
        // 2^(6/2) = 2^3 = 8
        let mut env = make_env();
        let base = Value::Integer(QInt::from(2i64));
        let exp = Value::Rational(QRat::from((6i64, 2i64))); // 6/2 = 3/1
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(8i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_rational_base_rational_exp() {
        // (1/2)^(4/2) = (1/2)^2 = 1/4
        let mut env = make_env();
        let base = Value::Rational(QRat::from((1i64, 2i64)));
        let exp = Value::Rational(QRat::from((4i64, 2i64))); // 4/2 = 2/1
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(r, QRat::from((1i64, 4i64)));
        } else {
            panic!("expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_jacobi_rational() {
        // JacobiProduct ^ (4/2) = scale exponents by 2
        let mut env = make_env();
        let base = Value::JacobiProduct(vec![(1, 5, 1)]);
        let exp = Value::Rational(QRat::from((4i64, 2i64))); // 4/2 = 2/1
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors, vec![(1, 5, 2)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_loop_exponent() {
        // Simulates for-loop scenario: n=3, q^(n*n) = q^9
        // In a for-loop, n*n with Rational n produces Rational(9/1)
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((9i64, 1i64)));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(9), QRat::one());
            assert_eq!(fps.coeff(0), QRat::zero());
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    // -- FractionalPowerSeries tests ------------------------------------------

    #[test]
    fn eval_pow_symbol_fractional_quarter() {
        // q^(1/4) -> FractionalPowerSeries { denom: 4, inner monomial at key 1 }
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((1i64, 4i64)));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 4);
            assert_eq!(inner.coeff(1), QRat::one());
            assert_eq!(inner.coeff(0), QRat::zero());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_symbol_fractional_third() {
        // q^(1/3) -> FractionalPowerSeries { denom: 3, inner monomial at key 1 }
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((1i64, 3i64)));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 3);
            assert_eq!(inner.coeff(1), QRat::one());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_pow_symbol_fractional_two_thirds() {
        // q^(2/3) -> FractionalPowerSeries { denom: 3, inner monomial at key 2 }
        let mut env = make_env();
        let base = Value::Symbol("q".to_string());
        let exp = Value::Rational(QRat::from((2i64, 3i64)));
        let result = eval_pow(base, exp, &mut env).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 3);
            assert_eq!(inner.coeff(2), QRat::one());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_div_series_by_fractional() {
        // (q + q^2) / q^(1/2) should give FractionalPowerSeries with denom=2
        // q -> key 2 in rescaled space; q^2 -> key 4.
        // Divide by monomial at key 1 (q^(1/2)): keys become 1 and 3.
        let mut env = make_env();
        let mut coeffs = BTreeMap::new();
        coeffs.insert(1, QRat::one());
        coeffs.insert(2, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(env.sym_q, coeffs, 20);
        let div_fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let result = eval_div(
            Value::Series(fps),
            Value::FractionalPowerSeries { inner: div_fps, denom: 2 },
            &mut env,
        ).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 2);
            // q / q^(1/2) = q^(1/2) -> key 1 in denom=2 space
            assert_eq!(inner.coeff(1), QRat::one());
            // q^2 / q^(1/2) = q^(3/2) -> key 3 in denom=2 space
            assert_eq!(inner.coeff(3), QRat::one());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_fractional_mul_simplifies_to_series() {
        // q^(1/2) * q^(1/2) should simplify to q (regular Series)
        let mut env = make_env();
        let a_inner = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let b_inner = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let result = eval_mul(
            Value::FractionalPowerSeries { inner: a_inner, denom: 2 },
            Value::FractionalPowerSeries { inner: b_inner, denom: 2 },
            &mut env,
        ).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(1), QRat::one());
            assert_eq!(fps.coeff(0), QRat::zero());
        } else {
            panic!("expected Series (simplified), got {:?}", result);
        }
    }

    #[test]
    fn eval_fractional_add() {
        // q^(1/2) + 2*q^(1/2) = 3*q^(1/2)
        let mut env = make_env();
        let a = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let b = FormalPowerSeries::monomial(env.sym_q, QRat::from((2i64, 1i64)), 1, POLYNOMIAL_ORDER);
        let result = eval_add(
            Value::FractionalPowerSeries { inner: a, denom: 2 },
            Value::FractionalPowerSeries { inner: b, denom: 2 },
            &mut env,
        ).unwrap();
        if let Value::FractionalPowerSeries { inner, denom } = result {
            assert_eq!(denom, 2);
            assert_eq!(inner.coeff(1), QRat::from((3i64, 1i64)));
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_fractional_negate() {
        let mut env = make_env();
        let inner = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let result = eval_negate(
            Value::FractionalPowerSeries { inner, denom: 4 },
            &mut env,
        ).unwrap();
        if let Value::FractionalPowerSeries { inner: neg_inner, denom } = result {
            assert_eq!(denom, 4);
            assert_eq!(neg_inner.coeff(1), -QRat::one());
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_fractional_scalar_mul() {
        let mut env = make_env();
        let inner = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, POLYNOMIAL_ORDER);
        let result = eval_mul(
            Value::Integer(QInt::from(3i64)),
            Value::FractionalPowerSeries { inner, denom: 4 },
            &mut env,
        ).unwrap();
        if let Value::FractionalPowerSeries { inner: res_inner, denom } = result {
            assert_eq!(denom, 4);
            assert_eq!(res_inner.coeff(1), QRat::from((3i64, 1i64)));
        } else {
            panic!("expected FractionalPowerSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_add_jacobi_product_errors() {
        let mut env = make_env();
        let left = Value::JacobiProduct(vec![(1, 5, 1)]);
        let right = Value::JacobiProduct(vec![(2, 5, 1)]);
        let result = eval_add(left, right, &mut env);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("jac2series"), "expected helpful message, got: {}", msg);
    }

    #[test]
    fn normalize_merges_same_factors() {
        let mut env = make_env();
        let left = Value::JacobiProduct(vec![(1, 5, 1)]);
        let right = Value::JacobiProduct(vec![(1, 5, 1)]);
        let result = eval_mul(left, right, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors, vec![(1, 5, 2)]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    #[test]
    fn normalize_removes_zero_exp() {
        let mut env = make_env();
        let left = Value::JacobiProduct(vec![(1, 5, 1)]);
        let right = Value::JacobiProduct(vec![(1, 5, 1)]);
        let result = eval_div(left, right, &mut env).unwrap();
        if let Value::JacobiProduct(factors) = result {
            assert_eq!(factors, vec![]);
        } else {
            panic!("expected JacobiProduct, got {:?}", result);
        }
    }

    // --- Dispatch: theta, jac2prod, jac2series (Task 2) ---

    #[test]
    fn dispatch_theta_numeric_z() {
        let mut env = make_env();
        // theta(1, q, 5) = sum(1^i * q^(i^2), i=-5..5)
        // i values: -5..5, i^2: 25,16,9,4,1,0,1,4,9,16,25
        // Only i^2 < 5: i in {-2,-1,0,1,2} -> q^{4,1,0,1,4}
        // coeff(0) = 1 (i=0), coeff(1) = 2 (i=+-1), coeff(4) = 2 (i=+-2)
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("theta", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(2), QRat::zero());
            assert_eq!(fps.coeff(3), QRat::zero());
            assert_eq!(fps.coeff(4), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_theta_monomial_z() {
        let mut env = make_env();
        // theta(q^2, q, 10) = sum(q^(2i + i^2), i=-10..10)
        // Exponent = 2i + i^2 = i(i+2). Only keep 0 <= exp < 10.
        // i=0: 0, i=1: 3, i=2: 8, i=-1: -1 (skip), i=-2: 0, i=3: 15 (skip)
        // i=-3: 3, i=-4: 8, i=-5: 15 (skip)
        // So coeff(0) = 2 (i=0 and i=-2), coeff(3) = 2 (i=1 and i=-3), coeff(8) = 2 (i=2 and i=-4)
        let sym_q = env.symbols.intern("q");
        let q_squared = FormalPowerSeries::monomial(sym_q, QRat::one(), 2, POLYNOMIAL_ORDER);
        let args = vec![
            Value::Series(q_squared),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("theta", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(3), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(8), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(1), QRat::zero());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_theta_symbol_z_warns() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("theta", &args, &mut env).unwrap();
        assert!(matches!(val, Value::None));
    }

    #[test]
    fn dispatch_jac2prod_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::JacobiProduct(vec![(1, 5, 1)]),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2prod", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // JAC(1,5) = (q;q^5)_inf = etaq(1,5,q,20)
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), -QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac2prod_wrong_type_errors() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(42i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let result = dispatch("jac2prod", &args, &mut env);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expected Jacobi product"), "expected Jacobi product error, got: {}", msg);
    }

    #[test]
    fn dispatch_jac2series_returns_series() {
        let mut env = make_env();
        let args = vec![
            Value::JacobiProduct(vec![(1, 5, 1)]),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2series", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), -QRat::one());
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac2series_matches_etaq() {
        let mut env = make_env();
        // jac2series(JAC(1,5), q, 20) should equal etaq(1, 5, q, 20)
        let jac_args = vec![
            Value::JacobiProduct(vec![(1, 5, 1)]),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let jac_val = dispatch("jac2series", &jac_args, &mut env).unwrap();

        let etaq_args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ];
        let etaq_val = dispatch("etaq", &etaq_args, &mut env).unwrap();

        if let (Value::Series(jac_fps), Value::Series(etaq_fps)) = (&jac_val, &etaq_val) {
            for k in 0..20 {
                assert_eq!(
                    jac_fps.coeff(k), etaq_fps.coeff(k),
                    "mismatch at q^{}: jac2series={}, etaq={}",
                    k, jac_fps.coeff(k), etaq_fps.coeff(k)
                );
            }
        } else {
            panic!("expected two Series");
        }
    }

    #[test]
    fn dispatch_jac2series_product() {
        let mut env = make_env();
        // jac2series(JAC(1,5)*JAC(4,5), q, 20) should equal etaq(1,5,q,20) * etaq(4,5,q,20)
        let jp = Value::JacobiProduct(vec![(1, 5, 1), (4, 5, 1)]);
        let jac_args = vec![
            jp,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let jac_val = dispatch("jac2series", &jac_args, &mut env).unwrap();

        // Compute expected via etaq
        let sym_q = env.sym_q;
        let e1 = qseries::etaq(1, 5, sym_q, 20);
        let e4 = qseries::etaq(4, 5, sym_q, 20);
        let expected = arithmetic::mul(&e1, &e4);

        if let Value::Series(jac_fps) = &jac_val {
            for k in 0..20 {
                assert_eq!(
                    jac_fps.coeff(k), expected.coeff(k),
                    "mismatch at q^{}: jac2series={}, expected={}",
                    k, jac_fps.coeff(k), expected.coeff(k)
                );
            }
        } else {
            panic!("expected Series");
        }
    }

    // --- jac2series 2-arg Garvan form tests ---

    #[test]
    fn dispatch_jac2series_2arg_basic() {
        let mut env = make_env();
        // 2-arg Garvan: jac2series(JAC(1,5), 20)
        // JAC(1,5) in Garvan convention = jacprod(1,5) = (q;q^5)(q^4;q^5)(q^5;q^5)
        let args = vec![
            Value::JacobiProduct(vec![(1, 5, 1)]),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2series", &args, &mut env).unwrap();
        // Should match jacprod(1, 5, sym_q, 20) = triple product
        let expected = qseries::jacprod(1, 5, env.sym_q, 20);
        if let Value::Series(fps) = &val {
            for k in 0..20 {
                assert_eq!(
                    fps.coeff(k), expected.coeff(k),
                    "2-arg jac2series JAC(1,5) mismatch at q^{}: got={}, expected={}",
                    k, fps.coeff(k), expected.coeff(k)
                );
            }
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac2series_2arg_jac0b() {
        let mut env = make_env();
        // 2-arg Garvan: jac2series(JAC(0,1), 20)
        // JAC(0,1) in Garvan convention = (q;q)_inf = etaq(1, 1, q, 20)
        let args = vec![
            Value::JacobiProduct(vec![(0, 1, 1)]),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2series", &args, &mut env).unwrap();
        let expected = qseries::etaq(1, 1, env.sym_q, 20);
        if let Value::Series(fps) = &val {
            // (q;q)_inf Euler function: 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + ...
            assert_eq!(fps.coeff(0), QRat::one(), "constant term should be 1");
            assert_eq!(fps.coeff(1), -QRat::one(), "q^1 should be -1");
            for k in 0..20 {
                assert_eq!(
                    fps.coeff(k), expected.coeff(k),
                    "JAC(0,1) mismatch at q^{}: got={}, expected={}",
                    k, fps.coeff(k), expected.coeff(k)
                );
            }
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac2series_3arg_unchanged() {
        let mut env = make_env();
        // 3-arg legacy: jac2series(JAC(1,5), q, 20) should still work as etaq
        let args = vec![
            Value::JacobiProduct(vec![(1, 5, 1)]),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2series", &args, &mut env).unwrap();
        // Legacy path uses etaq(1,5), NOT jacprod
        let expected = qseries::etaq(1, 5, env.sym_q, 20);
        if let Value::Series(fps) = &val {
            for k in 0..20 {
                assert_eq!(
                    fps.coeff(k), expected.coeff(k),
                    "3-arg legacy jac2series mismatch at q^{}",
                    k
                );
            }
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jac2series_2arg_product() {
        let mut env = make_env();
        // 2-arg Garvan: jac2series(JAC(1,5)*JAC(4,5), 20)
        // = jacprod(1,5) * jacprod(4,5)
        let args = vec![
            Value::JacobiProduct(vec![(1, 5, 1), (4, 5, 1)]),
            Value::Integer(QInt::from(20i64)),
        ];
        let val = dispatch("jac2series", &args, &mut env).unwrap();
        let jp1 = qseries::jacprod(1, 5, env.sym_q, 20);
        let jp4 = qseries::jacprod(4, 5, env.sym_q, 20);
        let expected = arithmetic::mul(&jp1, &jp4);
        if let Value::Series(fps) = &val {
            for k in 0..20 {
                assert_eq!(
                    fps.coeff(k), expected.coeff(k),
                    "2-arg product mismatch at q^{}: got={}, expected={}",
                    k, fps.coeff(k), expected.coeff(k)
                );
            }
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    // --- qs2jaccombo tests ---

    #[test]
    fn dispatch_qs2jaccombo_single_product() {
        let mut env = make_env();
        // (q;q)_inf is a single Jacobi product JAC(1,1)
        // jacprodmake should recognize this as a single product
        let sym_q = env.sym_q;
        let f = qseries::etaq(1, 1, sym_q, 30);
        let args = vec![
            Value::Series(f),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(30i64)),
        ];
        let val = dispatch("qs2jaccombo", &args, &mut env).unwrap();
        // Should find some JAC decomposition (either single product or linear combination)
        match &val {
            Value::String(s) => {
                assert!(s.contains("JAC"), "expected JAC in result: {}", s);
            }
            _ => {
                panic!("expected String result for Euler function, got {:?}", val);
            }
        }
    }

    #[test]
    fn dispatch_qs2jaccombo_returns_without_error() {
        let mut env = make_env();
        // qs2jaccombo should not error regardless of input -- it either finds
        // a decomposition (String) or returns the input series
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0, QRat::from((1i64, 1i64)));
        coeffs.insert(3, QRat::from((7i64, 1i64)));
        coeffs.insert(17, QRat::from((-3i64, 1i64)));
        let f = FormalPowerSeries::from_coeffs(sym_q, coeffs, 5);
        let args = vec![
            Value::Series(f),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("qs2jaccombo", &args, &mut env);
        assert!(val.is_ok(), "qs2jaccombo should not error: {:?}", val.err());
    }

    // --- Phase 38 analysis/discovery tests ---

    #[test]
    fn dispatch_lqdegree0_returns_min_order() {
        let mut env = make_env();
        // etaq(1,1,20) = prod (1-q^n) starts at q^0 (constant term 1)
        let eta = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("lqdegree0", &[eta], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(0i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_checkmult_partition_not_multiplicative() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(50i64))], &mut env).unwrap();
        let val = dispatch("checkmult", &[pgf, Value::Integer(QInt::from(30i64))], &mut env).unwrap();
        // partition function is NOT multiplicative
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(0i64));
        } else {
            panic!("expected Integer(0), got {:?}", val);
        }
    }

    #[test]
    fn dispatch_checkmult_with_yes_prints_all() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(50i64))], &mut env).unwrap();
        let val = dispatch("checkmult", &[
            pgf,
            Value::Integer(QInt::from(30i64)),
            Value::String("yes".to_string()),
        ], &mut env).unwrap();
        // Still returns 0 (not multiplicative), but prints all failures
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(0i64));
        } else {
            panic!("expected Integer(0), got {:?}", val);
        }
    }

    #[test]
    fn dispatch_checkprod_eta_nice_product() {
        let mut env = make_env();
        // etaq(1,1,30) = prod (1-q^n)^1 which is a nice product
        let eta = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(30i64)),
        ], &mut env).unwrap();
        let val = dispatch("checkprod", &[
            eta,
            Value::Integer(QInt::from(10i64)),  // M threshold
            Value::Integer(QInt::from(30i64)),  // Q order
        ], &mut env).unwrap();
        // Should be [a, 1] for a nice product
        if let Value::List(items) = &val {
            assert_eq!(items.len(), 2);
            // Second element should be 1 (nice)
            if let Value::Integer(code) = &items[1] {
                assert_eq!(*code, QInt::from(1i64), "expected nice product code 1");
            } else {
                panic!("expected Integer code, got {:?}", items[1]);
            }
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findprod_garvan_4arg() {
        let mut env = make_env();
        // Create two eta quotients as a simple test
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(30i64)),
        ], &mut env).unwrap();
        let e2 = dispatch("etaq", &[
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(30i64)),
        ], &mut env).unwrap();
        let val = dispatch("findprod", &[
            Value::List(vec![e1, e2]),
            Value::Integer(QInt::from(2i64)),   // T: max |coeff|
            Value::Integer(QInt::from(10i64)),  // M: max exponent threshold
            Value::Integer(QInt::from(30i64)),  // Q: truncation order
        ], &mut env).unwrap();
        // Should return a list (possibly empty, possibly with results)
        if let Value::List(results) = &val {
            // Each result should be [valuation, c1, c2, ...]
            for row in results {
                if let Value::List(items) = row {
                    assert!(items.len() >= 3, "each result should have valuation + k coefficients");
                } else {
                    panic!("expected List row, got {:?}", row);
                }
            }
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findprod_old_3arg_errors() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let result = dispatch("findprod", &[
            Value::List(vec![e1]),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env);
        assert!(result.is_err(), "old 3-arg findprod should now error (expects 4 args)");
    }

    // --- Control flow evaluation tests ---

    #[test]
    fn test_compare_integers() {
        let mut env = make_env();
        // 3 < 5 is true
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(5)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // 5 < 3 is false
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Integer(5)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));

        // 3 = 3 is true
        let node = AstNode::Compare {
            op: CompOp::Eq,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // 3 <> 3 is false
        let node = AstNode::Compare {
            op: CompOp::NotEq,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));

        // 5 > 3 is true
        let node = AstNode::Compare {
            op: CompOp::Greater,
            lhs: Box::new(AstNode::Integer(5)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // 3 <= 3 is true
        let node = AstNode::Compare {
            op: CompOp::LessEq,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(3)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // 3 >= 5 is false
        let node = AstNode::Compare {
            op: CompOp::GreaterEq,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Integer(5)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));
    }

    #[test]
    fn test_compare_rationals() {
        let mut env = make_env();
        // Set up 1/3 and 1/2 as variables
        env.set_var("a", Value::Rational(QRat::from((1i64, 3i64))));
        env.set_var("b", Value::Rational(QRat::from((1i64, 2i64))));

        // 1/3 < 1/2 is true
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("a".to_string())),
            rhs: Box::new(AstNode::Variable("b".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));
    }

    #[test]
    fn test_compare_mixed_int_rat() {
        let mut env = make_env();
        env.set_var("r", Value::Rational(QRat::from((3i64, 2i64))));

        // 1 < 3/2 is true
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Integer(1)),
            rhs: Box::new(AstNode::Variable("r".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // 3/2 < 2 is true (Rational vs Integer)
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("r".to_string())),
            rhs: Box::new(AstNode::Integer(2)),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));
    }

    #[test]
    fn test_compare_symbols_eq() {
        let mut env = make_env();
        // symbol "x" = symbol "x" is true (both undefined -> Symbol)
        let node = AstNode::Compare {
            op: CompOp::Eq,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Variable("x".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // symbol "x" = symbol "y" is false
        let node = AstNode::Compare {
            op: CompOp::Eq,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Variable("y".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));

        // symbol "x" <> symbol "y" is true
        let node = AstNode::Compare {
            op: CompOp::NotEq,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Variable("y".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // Ordering on symbols is an error
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Variable("y".to_string())),
        };
        assert!(eval_expr(&node, &mut env).is_err());
    }

    #[test]
    fn test_compare_bools_eq() {
        let mut env = make_env();
        env.set_var("t", Value::Bool(true));
        env.set_var("f", Value::Bool(false));

        // true = true is true
        let node = AstNode::Compare {
            op: CompOp::Eq,
            lhs: Box::new(AstNode::Variable("t".to_string())),
            rhs: Box::new(AstNode::Variable("t".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // true <> false is true
        let node = AstNode::Compare {
            op: CompOp::NotEq,
            lhs: Box::new(AstNode::Variable("t".to_string())),
            rhs: Box::new(AstNode::Variable("f".to_string())),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // Ordering on bools is an error
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("t".to_string())),
            rhs: Box::new(AstNode::Variable("f".to_string())),
        };
        assert!(eval_expr(&node, &mut env).is_err());
    }

    #[test]
    fn test_not_bool() {
        let mut env = make_env();
        env.set_var("t", Value::Bool(true));
        env.set_var("f", Value::Bool(false));

        // not true = false
        let node = AstNode::Not(Box::new(AstNode::Variable("t".to_string())));
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));

        // not false = true
        let node = AstNode::Not(Box::new(AstNode::Variable("f".to_string())));
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // not on non-bool is error
        let node = AstNode::Not(Box::new(AstNode::Integer(42)));
        assert!(eval_expr(&node, &mut env).is_err());
    }

    #[test]
    fn test_bool_and_short_circuit() {
        let mut env = make_env();
        // false and (1/0 which would error) returns false without evaluating rhs
        // We use an undefined function call as the rhs that would error if evaluated
        let node = AstNode::BoolOp {
            op: BoolBinOp::And,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // evaluates to false
            rhs: Box::new(AstNode::FuncCall {
                name: "NONEXISTENT_CRASH_FUNCTION".to_string(),
                args: vec![],
            }),
        };
        // This should return false, not error
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));

        // true and true = true
        let node = AstNode::BoolOp {
            op: BoolBinOp::And,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Integer(5)),
            }), // true
            rhs: Box::new(AstNode::Compare {
                op: CompOp::Eq,
                lhs: Box::new(AstNode::Integer(1)),
                rhs: Box::new(AstNode::Integer(1)),
            }), // true
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // true and false = false
        let node = AstNode::BoolOp {
            op: BoolBinOp::And,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Integer(5)),
            }), // true
            rhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));
    }

    #[test]
    fn test_bool_or_short_circuit() {
        let mut env = make_env();
        // true or (error) returns true without evaluating rhs
        let node = AstNode::BoolOp {
            op: BoolBinOp::Or,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Integer(5)),
            }), // true
            rhs: Box::new(AstNode::FuncCall {
                name: "NONEXISTENT_CRASH_FUNCTION".to_string(),
                args: vec![],
            }),
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // false or true = true
        let node = AstNode::BoolOp {
            op: BoolBinOp::Or,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
            rhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Integer(5)),
            }), // true
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(true)));

        // false or false = false
        let node = AstNode::BoolOp {
            op: BoolBinOp::Or,
            lhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
            rhs: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
        };
        assert!(matches!(eval_expr(&node, &mut env).unwrap(), Value::Bool(false)));
    }

    #[test]
    fn test_for_loop_basic() {
        let mut env = make_env();
        // for n from 1 to 5 do n^2 od -> returns 25 (last iteration)
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(5)),
            by: None,
            body: vec![Stmt {
                node: AstNode::BinOp {
                    op: BinOp::Pow,
                    lhs: Box::new(AstNode::Variable("n".to_string())),
                    rhs: Box::new(AstNode::Integer(2)),
                },
                terminator: Terminator::Implicit,
            }],
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(25i64));
        } else {
            panic!("expected Integer(25), got {:?}", val);
        }
    }

    #[test]
    fn test_for_loop_scoping() {
        let mut env = make_env();
        // Set n = 99 before the loop
        env.set_var("n", Value::Integer(QInt::from(99i64)));

        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(3)),
            by: None,
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        eval_expr(&node, &mut env).unwrap();

        // After loop, n should be restored to 99
        if let Some(Value::Integer(n)) = env.get_var("n") {
            assert_eq!(*n, QInt::from(99i64));
        } else {
            panic!("expected n to be restored to 99");
        }
    }

    #[test]
    fn test_for_loop_scoping_undefined_var() {
        let mut env = make_env();
        // n is undefined before loop
        assert!(env.get_var("n").is_none());

        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(3)),
            by: None,
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        eval_expr(&node, &mut env).unwrap();

        // After loop, n should still be undefined (removed)
        assert!(env.get_var("n").is_none());
    }

    #[test]
    fn test_for_loop_by() {
        let mut env = make_env();
        // for n from 0 to 10 by 2 do n od -> returns 10
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(0)),
            to: Box::new(AstNode::Integer(10)),
            by: Some(Box::new(AstNode::Integer(2))),
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(10i64));
        } else {
            panic!("expected Integer(10), got {:?}", val);
        }
    }

    #[test]
    fn test_for_loop_negative_step() {
        let mut env = make_env();
        // for n from 5 to 1 by -1 do n od -> returns 1
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(5)),
            to: Box::new(AstNode::Integer(1)),
            by: Some(Box::new(AstNode::Neg(Box::new(AstNode::Integer(1))))),
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(1i64));
        } else {
            panic!("expected Integer(1), got {:?}", val);
        }
    }

    #[test]
    fn test_for_loop_empty() {
        let mut env = make_env();
        // for n from 5 to 1 do n od -> None (zero iterations, step=1 but 5 > 1)
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(5)),
            to: Box::new(AstNode::Integer(1)),
            by: None,
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        let val = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(val, Value::None));
    }

    #[test]
    fn test_for_loop_zero_step_error() {
        let mut env = make_env();
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(5)),
            by: Some(Box::new(AstNode::Integer(0))),
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        assert!(eval_expr(&node, &mut env).is_err());
    }

    #[test]
    fn test_if_then_fi() {
        let mut env = make_env();
        // if true then 42 fi -> 42
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(1)),
                rhs: Box::new(AstNode::Integer(2)),
            }), // true
            then_body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: None,
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer(42), got {:?}", val);
        }
    }

    #[test]
    fn test_if_then_fi_false_no_else() {
        let mut env = make_env();
        // if false then 42 fi -> None
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Greater,
                lhs: Box::new(AstNode::Integer(1)),
                rhs: Box::new(AstNode::Integer(2)),
            }), // false
            then_body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: None,
        };
        let val = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(val, Value::None));
    }

    #[test]
    fn test_if_else() {
        let mut env = make_env();
        // if false then 1 else 2 fi -> 2
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Greater,
                lhs: Box::new(AstNode::Integer(1)),
                rhs: Box::new(AstNode::Integer(2)),
            }), // false
            then_body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: Some(vec![Stmt {
                node: AstNode::Integer(2),
                terminator: Terminator::Implicit,
            }]),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(2i64));
        } else {
            panic!("expected Integer(2), got {:?}", val);
        }
    }

    #[test]
    fn test_if_elif_else() {
        let mut env = make_env();
        // if 5 < 3 then 1 elif 5 > 3 then 2 else 3 fi -> 2 (elif branch)
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
            then_body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![
                (
                    AstNode::Compare {
                        op: CompOp::Greater,
                        lhs: Box::new(AstNode::Integer(5)),
                        rhs: Box::new(AstNode::Integer(3)),
                    }, // true
                    vec![Stmt {
                        node: AstNode::Integer(2),
                        terminator: Terminator::Implicit,
                    }],
                ),
            ],
            else_body: Some(vec![Stmt {
                node: AstNode::Integer(3),
                terminator: Terminator::Implicit,
            }]),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(2i64));
        } else {
            panic!("expected Integer(2), got {:?}", val);
        }

        // Test that else is reached when all conditions are false
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Integer(5)),
                rhs: Box::new(AstNode::Integer(3)),
            }), // false
            then_body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![
                (
                    AstNode::Compare {
                        op: CompOp::Less,
                        lhs: Box::new(AstNode::Integer(5)),
                        rhs: Box::new(AstNode::Integer(3)),
                    }, // false
                    vec![Stmt {
                        node: AstNode::Integer(2),
                        terminator: Terminator::Implicit,
                    }],
                ),
            ],
            else_body: Some(vec![Stmt {
                node: AstNode::Integer(3),
                terminator: Terminator::Implicit,
            }]),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(3i64));
        } else {
            panic!("expected Integer(3), got {:?}", val);
        }
    }

    #[test]
    fn test_if_integer_truthy() {
        let mut env = make_env();
        // if 1 then 42 fi -> 42 (nonzero integer is truthy)
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Integer(1)),
            then_body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: None,
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer(42), got {:?}", val);
        }

        // if 0 then 42 else 99 fi -> 99 (zero is falsy)
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Integer(0)),
            then_body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: Some(vec![Stmt {
                node: AstNode::Integer(99),
                terminator: Terminator::Implicit,
            }]),
        };
        let val = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(99i64));
        } else {
            panic!("expected Integer(99), got {:?}", val);
        }
    }

    #[test]
    fn test_return_top_level() {
        let mut env = make_env();
        // RETURN(5) at top level produces EarlyReturn error
        let node = AstNode::FuncCall {
            name: "RETURN".to_string(),
            args: vec![AstNode::Integer(5)],
        };
        let result = eval_expr(&node, &mut env);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should display as "Error: RETURN used outside of a procedure"
        let msg = format!("{}", err);
        assert!(msg.contains("RETURN"), "error message should mention RETURN: {}", msg);
        assert!(msg.contains("outside"), "error message should say 'outside': {}", msg);

        // Also verify the value is preserved in the EarlyReturn
        if let EvalError::EarlyReturn(val) = err {
            if let Value::Integer(n) = val {
                assert_eq!(n, QInt::from(5i64));
            } else {
                panic!("expected Integer(5) in EarlyReturn, got {:?}", val);
            }
        } else {
            panic!("expected EarlyReturn error, got {:?}", err);
        }
    }

    #[test]
    fn test_return_wrong_arg_count() {
        let mut env = make_env();
        // RETURN() with no args
        let node = AstNode::FuncCall {
            name: "RETURN".to_string(),
            args: vec![],
        };
        assert!(eval_expr(&node, &mut env).is_err());

        // RETURN(1, 2) with too many args
        let node = AstNode::FuncCall {
            name: "RETURN".to_string(),
            args: vec![AstNode::Integer(1), AstNode::Integer(2)],
        };
        assert!(eval_expr(&node, &mut env).is_err());
    }

    #[test]
    fn test_is_truthy() {
        // Bool true -> true
        assert_eq!(is_truthy(&Value::Bool(true)).unwrap(), true);
        // Bool false -> false
        assert_eq!(is_truthy(&Value::Bool(false)).unwrap(), false);
        // Integer 0 -> false
        assert_eq!(is_truthy(&Value::Integer(QInt::from(0i64))).unwrap(), false);
        // Integer 1 -> true
        assert_eq!(is_truthy(&Value::Integer(QInt::from(1i64))).unwrap(), true);
        // Integer -5 -> true (nonzero)
        assert_eq!(is_truthy(&Value::Integer(QInt::from(-5i64))).unwrap(), true);
        // Other types -> error
        assert!(is_truthy(&Value::String("hello".to_string())).is_err());
        assert!(is_truthy(&Value::None).is_err());
    }

    #[test]
    fn test_stmt_sequence() {
        let mut env = make_env();
        // Empty sequence returns None
        let val = eval_stmt_sequence(&[], &mut env).unwrap();
        assert!(matches!(val, Value::None));

        // Single statement
        let stmts = vec![Stmt {
            node: AstNode::Integer(42),
            terminator: Terminator::Semi,
        }];
        let val = eval_stmt_sequence(&stmts, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer(42)");
        }

        // Multiple statements: returns last
        let stmts = vec![
            Stmt {
                node: AstNode::Assign {
                    name: "x".to_string(),
                    value: Box::new(AstNode::Integer(10)),
                },
                terminator: Terminator::Semi,
            },
            Stmt {
                node: AstNode::Variable("x".to_string()),
                terminator: Terminator::Implicit,
            },
        ];
        let val = eval_stmt_sequence(&stmts, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(10i64));
        } else {
            panic!("expected Integer(10)");
        }
    }

    #[test]
    fn test_for_loop_accumulate() {
        let mut env = make_env();
        // Use a for loop that accumulates: for n from 1 to 5 do s := s + n od
        // First set s := 0
        env.set_var("s", Value::Integer(QInt::from(0i64)));

        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(5)),
            by: None,
            body: vec![Stmt {
                node: AstNode::Assign {
                    name: "s".to_string(),
                    value: Box::new(AstNode::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(AstNode::Variable("s".to_string())),
                        rhs: Box::new(AstNode::Variable("n".to_string())),
                    }),
                },
                terminator: Terminator::Implicit,
            }],
        };
        eval_expr(&node, &mut env).unwrap();

        // s should be 1+2+3+4+5 = 15
        if let Some(Value::Integer(n)) = env.get_var("s") {
            assert_eq!(*n, QInt::from(15i64));
        } else {
            panic!("expected s=15");
        }
    }

    #[test]
    fn test_compare_cross_type_error() {
        let mut env = make_env();
        env.set_var("b", Value::Bool(true));

        // Comparing bool and integer is a type error
        let node = AstNode::Compare {
            op: CompOp::Eq,
            lhs: Box::new(AstNode::Variable("b".to_string())),
            rhs: Box::new(AstNode::Integer(1)),
        };
        assert!(eval_expr(&node, &mut env).is_err());
    }

    // =======================================================
    // Procedure tests
    // =======================================================

    /// Helper: parse and evaluate a multi-statement string, returning the
    /// last value and a mutable reference to the environment.
    fn eval_input(input: &str, env: &mut Environment) -> Result<Value, EvalError> {
        let stmts = crate::parser::parse(input).expect("parse error");
        let mut result = Value::None;
        for stmt in &stmts {
            result = eval_expr(&stmt.node, env)?;
        }
        Ok(result)
    }

    #[test]
    fn test_proc_simple() {
        let mut env = make_env();
        let val = eval_input("f := proc(n) n*n; end; f(5)", &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(25i64));
        } else {
            panic!("expected Integer(25), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_local_scoping() {
        let mut env = make_env();
        let val = eval_input("f := proc(n) local k; k := n*n; k; end; f(5)", &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(25i64));
        } else {
            panic!("expected Integer(25), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_local_not_leaking() {
        let mut env = make_env();
        eval_input("f := proc(n) local k; k := n*n; k; end; f(5)", &mut env).unwrap();
        // k should not be in environment after procedure call
        assert!(env.get_var("k").is_none(), "local variable k should not leak into global scope");
    }

    #[test]
    fn test_proc_return_early() {
        let mut env = make_env();
        let val = eval_input("f := proc(n) RETURN(n*2); 999; end; f(5)", &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(10i64));
        } else {
            panic!("expected Integer(10), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_return_in_for_loop() {
        let mut env = make_env();
        let val = eval_input(
            "f := proc(n) for k from 1 to 100 do if k = n then RETURN(k*k) fi od end; f(7)",
            &mut env,
        ).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(49i64));
        } else {
            panic!("expected Integer(49), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_option_remember() {
        let mut env = make_env();
        eval_input("f := proc(n) option remember; n*n; end; f(5)", &mut env).unwrap();
        // Check memo table has entry after first call
        if let Some(Value::Procedure(proc_val)) = env.get_var("f") {
            let memo = proc_val.memo.borrow();
            assert!(!memo.is_empty(), "memo table should have entry after call");
        } else {
            panic!("expected f to be a Procedure");
        }
    }

    #[test]
    fn test_proc_memoized_fib() {
        let mut env = make_env();
        let val = eval_input(
            "fib := proc(n) option remember; if n <= 1 then RETURN(n) fi; fib(n-1) + fib(n-2); end; fib(10)",
            &mut env,
        ).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(55i64));
        } else {
            panic!("expected Integer(55), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_wrong_arg_count() {
        let mut env = make_env();
        eval_input("f := proc(n) n; end", &mut env).unwrap();
        let result = eval_input("f(1, 2)", &mut env);
        assert!(result.is_err(), "should error on wrong arg count");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects 1"), "error should mention expected args: {}", msg);
    }

    #[test]
    fn test_proc_shadows_builtin() {
        let mut env = make_env();
        let val = eval_input("numbpart := proc(n) n*2; end; numbpart(5)", &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(10i64));
        } else {
            panic!("expected Integer(10), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_empty_body() {
        let mut env = make_env();
        let val = eval_input("f := proc() end; f()", &mut env).unwrap();
        assert!(matches!(val, Value::None), "empty proc body should return None");
    }

    #[test]
    fn test_proc_multiple_stmts() {
        let mut env = make_env();
        let val = eval_input(
            "f := proc(n) local a, b; a := n; b := a + 1; b; end; f(5)",
            &mut env,
        ).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(6i64));
        } else {
            panic!("expected Integer(6), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_nested_for_if() {
        let mut env = make_env();
        let val = eval_input(
            "f := proc(n) local s; s := 0; for k from 1 to n do if k > 2 then s := s + k fi od; s; end; f(5)",
            &mut env,
        ).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(12i64)); // 3 + 4 + 5 = 12
        } else {
            panic!("expected Integer(12), got {:?}", val);
        }
    }

    #[test]
    fn test_proc_restore_on_error() {
        let mut env = make_env();
        env.set_var("x", Value::Integer(QInt::from(99i64)));
        // Procedure that takes x as param, then errors on unknown function
        eval_input("f := proc(x) badfunction(); end", &mut env).unwrap();
        let _result = eval_input("f(1)", &mut env);
        // x should be restored to 99 regardless of error
        if let Some(Value::Integer(n)) = env.get_var("x") {
            assert_eq!(*n, QInt::from(99i64));
        } else {
            panic!("x should be restored to 99 after proc error");
        }
    }

    // =======================================================
    // Lambda (arrow) tests
    // =======================================================

    #[test]
    fn test_eval_lambda_simple() {
        let mut env = make_env();
        let val = eval_input("F := q -> q + 1; F(5)", &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(6i64));
        } else {
            panic!("expected Integer(6), got {:?}", val);
        }
    }

    #[test]
    fn test_eval_lambda_with_series() {
        let mut env = make_env();
        let val = eval_input("F := q -> q^2; F(q)", &mut env).unwrap();
        if let Value::Series(fps) = val {
            // q^2 should have a monomial at exponent 2
            assert!(fps.coeff(2) != QRat::zero(), "expected nonzero coeff at q^2");
        } else {
            panic!("expected Series, got {:?}", val);
        }
    }

    #[test]
    fn test_eval_lambda_arity_error() {
        let mut env = make_env();
        let result = eval_input("F := q -> q; F(1, 2)", &mut env);
        assert!(result.is_err(), "expected arity error for F(1,2)");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects 1") || msg.contains("arity"), "got: {}", msg);
    }

    #[test]
    fn test_eval_lambda_procedure_display() {
        let mut env = make_env();
        let val = eval_input("q -> q^2", &mut env).unwrap();
        if let Value::Procedure(proc) = val {
            assert_eq!(proc.params, vec!["q"]);
        } else {
            panic!("expected Procedure, got {:?}", val);
        }
    }

    // --- floor() tests ---

    #[test]
    fn dispatch_floor_integer() {
        let mut env = make_env();
        let val = dispatch("floor", &[Value::Integer(QInt::from(7i64))], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_floor_rational_positive() {
        let mut env = make_env();
        let val = dispatch("floor", &[Value::Rational(QRat::from((7i64, 3i64)))], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(2i64)); // floor(7/3) = 2
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_floor_rational_negative() {
        let mut env = make_env();
        let val = dispatch("floor", &[Value::Rational(QRat::from((-7i64, 3i64)))], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(-3i64)); // floor(-7/3) = -3
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_floor_exact_integer_rational() {
        let mut env = make_env();
        let val = dispatch("floor", &[Value::Rational(QRat::from((6i64, 3i64)))], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(2i64)); // floor(6/3) = floor(2) = 2
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_floor_wrong_type() {
        let mut env = make_env();
        let fps = FormalPowerSeries::monomial(env.sym_q, QRat::one(), 1, 20);
        let result = dispatch("floor", &[Value::Series(fps)], &mut env);
        assert!(result.is_err());
    }

    // --- legendre() tests ---

    #[test]
    fn dispatch_legendre_basic() {
        let mut env = make_env();
        let val = dispatch("legendre", &[
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(-1i64)); // legendre(2, 5) = -1
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_legendre_zero() {
        let mut env = make_env();
        let val = dispatch("legendre", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(0i64)); // legendre(5, 5) = 0
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_legendre_one() {
        let mut env = make_env();
        let val = dispatch("legendre", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(7i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(1i64)); // legendre(1, 7) = 1
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_legendre_invalid_p() {
        let mut env = make_env();
        let result = dispatch("legendre", &[
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(4i64)),
        ], &mut env);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("odd prime"), "got: {}", msg);
    }

    #[test]
    fn dispatch_legendre_alias_l() {
        assert_eq!(resolve_alias("L"), "legendre".to_string());
    }

    // --- series() tests ---

    #[test]
    fn dispatch_series_truncate_down() {
        let mut env = make_env();
        // Create (q;q)_inf to O(q^20)
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Infinity,
            Value::Integer(QInt::from(20i64)),
        ];
        let series_val = dispatch("aqprod", &args, &mut env).unwrap();
        if let Value::Series(ref fps) = series_val {
            assert_eq!(fps.truncation_order(), 20);
        } else {
            panic!("expected series");
        }
        // Now truncate to T=10
        let trunc_args = vec![
            series_val,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let result = dispatch("series", &trunc_args, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 10);
            for (&k, _) in fps.iter() {
                assert!(k < 10, "found coefficient at k={} >= 10", k);
            }
        } else {
            panic!("expected series");
        }
    }

    #[test]
    fn dispatch_series_truncate_up_capped() {
        let mut env = make_env();
        // Create (q;q)_inf to O(q^10)
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Infinity,
            Value::Integer(QInt::from(10i64)),
        ];
        let series_val = dispatch("aqprod", &args, &mut env).unwrap();
        // Try to truncate to T=100 -> should cap at 10
        let trunc_args = vec![
            series_val,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(100i64)),
        ];
        let result = dispatch("series", &trunc_args, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 10, "should be min(100, 10) = 10");
        } else {
            panic!("expected series");
        }
    }

    #[test]
    fn dispatch_series_jacobi_product() {
        let mut env = make_env();
        // Create JAC(1,5) * JAC(4,5)
        let jac1 = dispatch("JAC", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        let jac4 = dispatch("JAC", &[
            Value::Integer(QInt::from(4i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        // Multiply them
        let product = eval_mul(jac1, jac4, &mut env).unwrap();
        assert!(matches!(product, Value::JacobiProduct(_)));
        // Call series on JacobiProduct
        let args = vec![
            product,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(15i64)),
        ];
        let result = dispatch("series", &args, &mut env).unwrap();
        assert!(matches!(result, Value::Series(_)));
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 15);
        }
    }

    #[test]
    fn dispatch_series_integer() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(3i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let result = dispatch("series", &args, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 10);
            // Should have constant term 3
            let coeff0 = fps.iter().find(|(&k, _)| k == 0);
            assert!(coeff0.is_some(), "should have constant term");
            let (_, val) = coeff0.unwrap();
            assert_eq!(*val, QRat::from((3i64, 1i64)));
        } else {
            panic!("expected series");
        }
    }

    // --- expand() tests ---

    #[test]
    fn dispatch_expand_series_identity() {
        let mut env = make_env();
        // Create a series
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Infinity,
            Value::Integer(QInt::from(10i64)),
        ];
        let series_val = dispatch("aqprod", &args, &mut env).unwrap();
        let expand_result = dispatch("expand", &[series_val.clone()], &mut env).unwrap();
        // Should return same series
        assert!(matches!(expand_result, Value::Series(_)));
    }

    #[test]
    fn dispatch_expand_jacobi_product() {
        let mut env = make_env();
        let jac1 = dispatch("JAC", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        let jac4 = dispatch("JAC", &[
            Value::Integer(QInt::from(4i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        let product = eval_mul(jac1, jac4, &mut env).unwrap();
        let result = dispatch("expand", &[product], &mut env).unwrap();
        assert!(matches!(result, Value::Series(_)));
    }

    #[test]
    fn dispatch_expand_integer_identity() {
        let mut env = make_env();
        let result = dispatch("expand", &[Value::Integer(QInt::from(3i64))], &mut env).unwrap();
        assert!(matches!(result, Value::Integer(_)));
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(3i64));
        }
    }

    #[test]
    fn dispatch_expand_3arg_jacobi() {
        let mut env = make_env();
        let jac1 = dispatch("JAC", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        let args = vec![
            jac1,
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(15i64)),
        ];
        let result = dispatch("expand", &args, &mut env).unwrap();
        assert!(matches!(result, Value::Series(_)));
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 15);
        }
    }

    #[test]
    fn dispatch_expand_wrong_type() {
        let mut env = make_env();
        let result = dispatch("expand", &[Value::Bool(true)], &mut env);
        assert!(result.is_err());
    }

    // --- factor() dispatch tests ---

    #[test]
    fn dispatch_factor_1_minus_q6() {
        let mut env = make_env();
        // Build 1 - q^6 as an exact polynomial FPS
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0i64, QRat::one());
        coeffs.insert(6i64, QRat::from((-1i64, 1i64)));
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
        let result = dispatch("factor", &[Value::Series(fps)], &mut env).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("1-q") || s.contains("-1+q") || s.contains("q-1"),
                "should contain (q-1) factor: got {}", s);
            assert!(s.contains("q+1") || s.contains("1+q"),
                "should contain (q+1) factor: got {}", s);
            assert!(s.contains("q^2"),
                "should contain degree-2 factors: got {}", s);
        } else {
            panic!("factor should return Value::String, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_factor_truncated_series_error() {
        let mut env = make_env();
        // Build a truncated series (truncation_order != POLYNOMIAL_ORDER)
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0i64, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, 10);
        let result = dispatch("factor", &[Value::Series(fps)], &mut env);
        assert!(result.is_err(), "factor of truncated series should error");
        if let Err(EvalError::Other(msg)) = result {
            assert!(msg.contains("truncated"), "error should mention truncated: got {}", msg);
        }
    }

    #[test]
    fn dispatch_factor_constant() {
        let mut env = make_env();
        let result = dispatch("factor", &[Value::Integer(QInt::from(42i64))], &mut env).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("42"), "constant factoring should show the constant: got {}", s);
        } else {
            panic!("factor should return Value::String");
        }
    }

    // --- subs() substitution tests ---

    #[test]
    fn subs_q_equals_1_sums_coefficients() {
        // subs(q=1, 1 + q + q^2) = 3
        let mut env = make_env();
        let stmts = crate::parser::parse("subs(q=1, 1 + q + q^2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(3i64));
        } else {
            panic!("expected Integer(3), got {:?}", result);
        }
    }

    #[test]
    fn subs_q_equals_0_returns_constant_term() {
        // subs(q=0, 1 + q + q^2) = 1
        let mut env = make_env();
        let stmts = crate::parser::parse("subs(q=0, 1 + q + q^2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(1i64));
        } else {
            panic!("expected Integer(1), got {:?}", result);
        }
    }

    #[test]
    fn subs_q_equals_half_evaluates_rational() {
        // subs(q=1/2, 1 + q + q^2) = 1 + 1/2 + 1/4 = 7/4
        let mut env = make_env();
        let stmts = crate::parser::parse("subs(q=1/2, 1 + q + q^2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(r, QRat::from((7i64, 4i64)));
        } else {
            panic!("expected Rational(7/4), got {:?}", result);
        }
    }

    #[test]
    fn subs_q_squared_scales_exponents() {
        // subs(q=q^2, 1 + q + q^2) -> 1 + q^2 + q^4
        let mut env = make_env();
        let stmts = crate::parser::parse("subs(q=q^2, 1 + q + q^2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(0), QRat::one(), "constant term should be 1");
            assert_eq!(fps.coeff(2), QRat::one(), "q^2 coefficient should be 1");
            assert_eq!(fps.coeff(4), QRat::one(), "q^4 coefficient should be 1");
            assert_eq!(fps.coeff(1), QRat::zero(), "q^1 coefficient should be 0");
            assert_eq!(fps.coeff(3), QRat::zero(), "q^3 coefficient should be 0");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn subs_q_squared_scales_truncation_order() {
        // For a truncated series, q->q^2 should double the truncation order
        let mut env = make_env();
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0i64, QRat::one());
        coeffs.insert(1i64, QRat::one());
        coeffs.insert(2i64, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, 10);
        let target = Value::Series(fps);

        // sub_value: q^2
        let sub_fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 2, POLYNOMIAL_ORDER);
        let sub_value = Value::Series(sub_fps);

        let result = perform_substitution("q", sub_value, target, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), 20, "truncation order should be doubled");
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(2), QRat::one());
            assert_eq!(fps.coeff(4), QRat::one());
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn subs_q_squared_polynomial_preserves_polynomial_order() {
        // For exact polynomials (POLYNOMIAL_ORDER), q->q^2 should preserve POLYNOMIAL_ORDER
        let mut env = make_env();
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0i64, QRat::one());
        coeffs.insert(1i64, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
        let target = Value::Series(fps);

        let sub_fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 2, POLYNOMIAL_ORDER);
        let sub_value = Value::Series(sub_fps);

        let result = perform_substitution("q", sub_value, target, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.truncation_order(), POLYNOMIAL_ORDER);
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn subs_on_non_series_returns_unchanged() {
        let mut env = make_env();
        let target = Value::Integer(QInt::from(42i64));
        let sub_value = Value::Integer(QInt::from(1i64));
        let result = perform_substitution("q", sub_value, target, &mut env).unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer(42), got {:?}", result);
        }
    }

    #[test]
    fn subs_wrong_arg_count_errors() {
        let mut env = make_env();
        // subs with 1 arg should error
        let stmts = crate::parser::parse("subs(q=1)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env);
        assert!(result.is_err() || matches!(result, Ok(Some(Value::None))),
            "subs with 1 arg should error");
    }

    #[test]
    fn subs_without_equals_errors() {
        let mut env = make_env();
        // subs(1, 2) -- first arg is not var=val
        let stmts = crate::parser::parse("subs(1, 2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env);
        assert!(result.is_err(), "subs without = in first arg should error");
    }

    #[test]
    fn subs_mismatched_variable_returns_unchanged() {
        // subs(x=1, 1+q+q^2) where series is in q, not x -> return unchanged
        let mut env = make_env();
        let sym_q = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0i64, QRat::one());
        coeffs.insert(1i64, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs.clone(), POLYNOMIAL_ORDER);
        let target = Value::Series(fps);
        let sub_value = Value::Integer(QInt::from(1i64));

        let result = perform_substitution("x", sub_value, target, &mut env).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::one());
        } else {
            panic!("expected Series unchanged, got {:?}", result);
        }
    }

    // --- BivariateSeries arithmetic tests ---

    fn make_test_bivariate(env: &mut Environment) -> Value {
        use qsym_core::series::bivariate::BivariateSeries;
        let sym_q = env.symbols.intern("q");
        // q*z + O(q^10)
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 10);
        let bs = BivariateSeries::from_single_term("z".to_string(), 1, fps);
        Value::BivariateSeries(bs)
    }

    #[test]
    fn eval_bivariate_type_name() {
        let mut env = make_env();
        let val = make_test_bivariate(&mut env);
        assert_eq!(val.type_name(), "bivariate_series");
    }

    #[test]
    fn eval_bivariate_negate() {
        let mut env = make_env();
        let val = make_test_bivariate(&mut env);
        let neg = eval_negate(val, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = neg {
            let z1 = bs.terms.get(&1).unwrap();
            assert_eq!(z1.coeff(1), -QRat::one());
        } else {
            panic!("expected BivariateSeries, got {:?}", neg);
        }
    }

    #[test]
    fn eval_bivariate_add() {
        let mut env = make_env();
        let a = make_test_bivariate(&mut env);
        let b = make_test_bivariate(&mut env);
        let sum = eval_add(a, b, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = sum {
            let z1 = bs.terms.get(&1).unwrap();
            assert_eq!(z1.coeff(1), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected BivariateSeries, got {:?}", sum);
        }
    }

    #[test]
    fn eval_bivariate_mul_scalar() {
        let mut env = make_env();
        let bv = make_test_bivariate(&mut env);
        let three = Value::Integer(QInt::from(3i64));
        let result = eval_mul(bv, three, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = result {
            let z1 = bs.terms.get(&1).unwrap();
            assert_eq!(z1.coeff(1), QRat::from((3i64, 1i64)));
        } else {
            panic!("expected BivariateSeries, got {:?}", result);
        }
    }

    #[test]
    fn eval_bivariate_sub() {
        let mut env = make_env();
        let a = make_test_bivariate(&mut env);
        let b = make_test_bivariate(&mut env);
        let diff = eval_sub(a, b, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = diff {
            assert!(bs.is_zero(), "expected zero after subtracting identical series");
        } else {
            panic!("expected BivariateSeries, got {:?}", diff);
        }
    }

    // --- Bivariate tripleprod/quinprod dispatch tests ---

    #[test]
    fn dispatch_tripleprod_bivariate_basic() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("tripleprod", &args, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = val {
            // n=0: (-1)^0 * z^0 * q^0 = 1, so z^0 coefficient at q^0 should be 1
            let z0 = bs.terms.get(&0).unwrap();
            assert_eq!(z0.coeff(0), QRat::one(), "constant term z^0 q^0 should be 1");
        } else {
            panic!("expected BivariateSeries, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_tripleprod_bivariate_preserves_univariate() {
        let mut env = make_env();
        // When z and q are the SAME symbol, fall through to monomial path
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("tripleprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)),
            "tripleprod(q, q, 10) should be univariate Series, got {:?}", val.type_name());
    }

    #[test]
    fn dispatch_quinprod_bivariate_basic() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::BivariateSeries(_)),
            "quinprod(z, q, 10) should be BivariateSeries, got {:?}", val.type_name());
    }

    #[test]
    fn dispatch_quinprod_bivariate_preserves_univariate() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)),
            "quinprod(q, q, 10) should be univariate Series, got {:?}", val.type_name());
    }

    // --- quinprod identity mode tests ---

    #[test]
    fn dispatch_quinprod_prodid() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Symbol("prodid".to_string()),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        if let Value::String(s) = &val {
            assert!(s.contains("(z"), "prodid should contain (z, got: {}", s);
            assert!(s.contains("(q,q)_inf"), "prodid should contain (q,q)_inf, got: {}", s);
            assert!(s.contains("(-z,q)_inf"), "prodid should contain (-z,q)_inf, got: {}", s);
        } else {
            panic!("expected String, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_quinprod_seriesid() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Symbol("seriesid".to_string()),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        if let Value::String(s) = &val {
            assert!(s.contains("sum"), "seriesid should contain 'sum', got: {}", s);
            assert!(s.contains("3*m"), "seriesid should contain '3*m', got: {}", s);
            // Also has the product side
            assert!(s.contains("(q,q)_inf"), "seriesid should contain product side too, got: {}", s);
        } else {
            panic!("expected String, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_quinprod_numeric_unchanged() {
        let mut env = make_env();
        // quinprod(z, q, 10) with z as a symbol and third arg as integer
        // should still produce a BivariateSeries
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("quinprod", &args, &mut env).unwrap();
        assert!(matches!(val, Value::BivariateSeries(_)),
            "quinprod(z, q, 10) should still produce BivariateSeries, got {:?}", val.type_name());
    }

    #[test]
    fn tripleprod_bivariate_sign_convention_validation() {
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc: i64 = 50;
        // Safe comparison bound: only compare coefficients well below the truncation
        // boundary, since evaluating the bivariate at z = c*q^m shifts truncation.
        let safe_bound: i64 = 15;

        // Cross-validate by evaluating bivariate at z = -q^m (coefficient -1).
        // z = q^m (coefficient 1) causes product zeros due to (q/z;q)_inf having
        // a zero factor at z = q^k for any integer k. Using z = -q^m avoids this.
        for m in [1i64, 2, 3] {
            let bv = compute_tripleprod_bivariate("z", sym_q, trunc);

            // Evaluate bivariate at z = -q^m: z^n = (-q^m)^n = (-1)^n * q^{mn}
            let mut eval_result = FormalPowerSeries::zero(sym_q, trunc);
            for (&z_exp, fps) in &bv.terms {
                let sign = if z_exp % 2 == 0 { QRat::one() } else { -QRat::one() };
                let shifted = arithmetic::shift(fps, z_exp * m);
                for (&k, c) in shifted.iter() {
                    if k >= 0 && k < trunc {
                        let old = eval_result.coeff(k);
                        eval_result.set_coeff(k, old + c.clone() * sign.clone());
                    }
                }
            }

            // Compare against numeric tripleprod with z = -q^m
            let monomial = QMonomial::new(-QRat::one(), m);
            let numeric = qseries::tripleprod(&monomial, sym_q, trunc);
            for k in 0..safe_bound {
                assert_eq!(
                    eval_result.coeff(k), numeric.coeff(k),
                    "tripleprod bivariate mismatch at q^{} for z=-q^{}", k, m
                );
            }
        }
    }

    #[test]
    fn quinprod_bivariate_validation() {
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc: i64 = 20;

        // Direct verification of quinprod bivariate coefficients against the sum formula:
        // quinprod(z, q, T) = sum_m (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
        let bv = compute_quinprod_bivariate("z", sym_q, trunc);

        // Each m contributes: +1 at z^{3m} and -1 at z^{-3m-1}, both at q^{m(3m+1)/2}.
        // Verify specific terms.

        // m=0: q_exp=0, z^0 gets +1*q^0, z^{-1} gets -1*q^0
        let z0 = bv.terms.get(&0).unwrap();
        assert_eq!(z0.coeff(0), QRat::one(), "z^0 coeff at q^0 should be 1");
        assert_eq!(z0.iter().count(), 1, "z^0 should have only one q-term");

        let zm1 = bv.terms.get(&-1).unwrap();
        assert_eq!(zm1.coeff(0), -QRat::one(), "z^(-1) coeff at q^0 should be -1");

        // m=1: q_exp=2, z^3 gets +1*q^2, z^(-4) gets -1*q^2
        let z3 = bv.terms.get(&3).unwrap();
        assert_eq!(z3.coeff(2), QRat::one(), "z^3 coeff at q^2 should be 1");

        let zm4 = bv.terms.get(&-4).unwrap();
        assert_eq!(zm4.coeff(2), -QRat::one(), "z^(-4) coeff at q^2 should be -1");

        // m=-1: q_exp=(-1)(-2)/2=1, z^(-3) gets +1*q^1, z^2 gets -1*q^1
        let zm3 = bv.terms.get(&-3).unwrap();
        assert_eq!(zm3.coeff(1), QRat::one(), "z^(-3) coeff at q^1 should be 1");

        let z2 = bv.terms.get(&2).unwrap();
        assert_eq!(z2.coeff(1), -QRat::one(), "z^2 coeff at q^1 should be -1");

        // m=2: q_exp=2*(7)/2=7, z^6 gets +1*q^7, z^(-7) gets -1*q^7
        let z6 = bv.terms.get(&6).unwrap();
        assert_eq!(z6.coeff(7), QRat::one(), "z^6 coeff at q^7 should be 1");

        let zm7 = bv.terms.get(&-7).unwrap();
        assert_eq!(zm7.coeff(7), -QRat::one(), "z^(-7) coeff at q^7 should be -1");

        // Verify that each z-exponent has exactly one nonzero q-coefficient
        // (since each z-exponent appears from exactly one m value in the sum)
        for (&z_exp, fps) in &bv.terms {
            assert_eq!(fps.iter().count(), 1,
                "z^{} should have exactly one q-term, got {}", z_exp, fps.iter().count());
        }
    }

    #[test]
    fn tripleprod_bivariate_symmetry() {
        let env = make_env();
        let sym_q = env.sym_q;
        let bv = compute_tripleprod_bivariate("z", sym_q, 20);

        // Each z^n coefficient should be (-1)^n * q^{n(n-1)/2} (a single monomial).
        for (&z_exp, fps) in &bv.terms {
            let q_exp = z_exp * (z_exp - 1) / 2;
            let expected_sign = if z_exp % 2 == 0 { QRat::one() } else { -QRat::one() };

            // Check that only q_exp has a nonzero coefficient
            let nonzero_count = fps.iter().count();
            assert_eq!(nonzero_count, 1,
                "z^{} should have exactly one q-term, got {}", z_exp, nonzero_count);
            assert_eq!(fps.coeff(q_exp), expected_sign,
                "z^{} coefficient at q^{} should be {:?}", z_exp, q_exp, expected_sign);
        }
    }

    #[test]
    fn bivariate_tripleprod_arithmetic() {
        let env = make_env();
        let sym_q = env.sym_q;
        let t1 = compute_tripleprod_bivariate("z", sym_q, 10);
        let t2 = compute_tripleprod_bivariate("z", sym_q, 10);

        // t1 + t2 should equal 2 * t1
        let sum = bv::bivariate_add(&t1, &t2);
        let doubled = bv::bivariate_scalar_mul(&QRat::from((2i64, 1i64)), &t1);
        assert_eq!(sum, doubled, "t1 + t2 should equal 2*t1");

        // t1 - t1 should be zero
        let diff = bv::bivariate_sub(&t1, &t1);
        assert!(diff.is_zero(), "t1 - t1 should be zero");
    }

    // --- Winquist bivariate tests ---

    #[test]
    fn dispatch_winquist_one_symbolic_a() {
        let mut env = make_env();
        // winquist(z, 2*q, q, 10) where z is symbolic and b=2*q avoids product zeros
        // (b=q^m with coeff=1 causes zero factors in the Winquist product)
        let sym_q = env.sym_q;
        let b_fps = FormalPowerSeries::monomial(sym_q, QRat::from((2i64, 1i64)), 1, POLYNOMIAL_ORDER);
        let args = vec![
            Value::Symbol("z".to_string()),
            Value::Series(b_fps),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = &val {
            assert!(!bs.is_zero(), "bivariate winquist should have nonzero terms");
            assert_eq!(bs.outer_variable(), "z");
        } else {
            panic!("expected BivariateSeries, got {:?}", val.type_name());
        }
    }

    #[test]
    fn dispatch_winquist_one_symbolic_b() {
        let mut env = make_env();
        // winquist(2*q, z, q, 10) where z is symbolic in position 1
        let sym_q = env.sym_q;
        let a_fps = FormalPowerSeries::monomial(sym_q, QRat::from((2i64, 1i64)), 1, POLYNOMIAL_ORDER);
        let args = vec![
            Value::Series(a_fps),
            Value::Symbol("z".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        if let Value::BivariateSeries(bs) = &val {
            assert!(!bs.is_zero(), "bivariate winquist (b symbolic) should have nonzero terms");
            assert_eq!(bs.outer_variable(), "z");
        } else {
            panic!("expected BivariateSeries, got {:?}", val.type_name());
        }
    }

    #[test]
    fn pochhammer_bivariate_basic() {
        // Verify compute_pochhammer_bivariate for a simple case:
        // (z; q)_inf truncated to O(q^5) should match the product form.
        // (z; q)_inf = prod_{k>=0}(1 - z*q^k) = 1 - z - z*q + z^2*q + ...
        let env = make_env();
        let sym_q = env.sym_q;

        let bv = compute_pochhammer_bivariate("z", &QRat::one(), 1, 0, sym_q, 5);
        // At z^0: should be 1
        assert_eq!(bv.terms.get(&0).unwrap().coeff(0), QRat::one(),
            "z^0 q^0 should be 1");
        // At z^1: should be -1 - q - q^2 - q^3 - q^4
        // Actually (z;q)_inf = prod(1-z*q^k): the z^1 coefficient comes from picking
        // the z-term from exactly one factor. The coefficient of z^1 is:
        // sum_{k=0}^{4} (-1)*q^k = -1 - q - q^2 - q^3 - q^4
        let z1 = bv.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(0), -QRat::one(), "z^1 q^0 should be -1");
        assert_eq!(z1.coeff(1), -QRat::one(), "z^1 q^1 should be -1");
    }

    #[test]
    fn pochhammer_bivariate_negative_offset() {
        // Verify (z*q^{-1}; q)_inf = prod_{k>=0}(1 - z*q^{k-1})
        // = (1 - z/q)(1 - z)(1 - z*q)(1 - z*q^2)...
        // After conversion to true coords, the q^{-1} term from the first factor
        // is dropped (FPS only stores non-negative exponents). But all non-negative
        // terms should be present.
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc = 5i64;

        let bv = compute_pochhammer_bivariate("z", &QRat::one(), 1, -1, sym_q, trunc);

        // z^0 should be 1
        assert_eq!(bv.terms.get(&0).unwrap().coeff(0), QRat::one(),
            "z^0 q^0 should be 1");
        // z^1: picking the z-term from factor k gives -q^{k-1}. For k=0: -q^{-1} (dropped).
        // For k >= 1: -q^{k-1}. So stored coefficients are -1 at q^0, -1 at q^1, etc.
        let z1 = bv.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(0), -QRat::one(), "z^1 q^0 should be -1");
        assert_eq!(z1.coeff(1), -QRat::one(), "z^1 q^1 should be -1");
    }

    #[test]
    fn winquist_bivariate_zero_offset() {
        // winquist(z, 2, q, 10) -- b has bp=0, no negative offsets.
        // Cross-validate at z=-1 against numeric winquist(-1, 2, q, 10).
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc: i64 = 10;

        let b_mono = QMonomial::new(QRat::from((2i64, 1i64)), 0); // b = 2 (constant)
        let bv = compute_winquist_one_symbolic("z", &b_mono, sym_q, trunc);

        // Evaluate at z = -1
        let mut evaluated = FormalPowerSeries::zero(sym_q, trunc);
        for (&z_exp, fps) in &bv.terms {
            let sign = if z_exp % 2 == 0 { QRat::one() } else { -QRat::one() };
            for (&q_pow, coeff) in fps.iter() {
                if q_pow >= 0 && q_pow < trunc {
                    let old = evaluated.coeff(q_pow);
                    evaluated.set_coeff(q_pow, old + sign.clone() * coeff.clone());
                }
            }
        }

        // Numeric: winquist(-1, 2, q, 10)
        let a_mono = QMonomial::new(-QRat::one(), 0);
        let numeric = qseries::winquist(&a_mono, &b_mono, sym_q, trunc);

        for k in 0..8 {
            assert_eq!(evaluated.coeff(k), numeric.coeff(k),
                "q^{} mismatch", k);
        }
    }

    #[test]
    fn winquist_bivariate_validation() {
        // Verify bivariate winquist(z, 2*q, q, T) evaluated at z = -1 matches
        // numeric winquist(-1, 2*q, q, T). Using z=-1 avoids q-shift complexity
        // in the evaluation step (z^n = (-1)^n, no q-exponent contribution from z).
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc: i64 = 30;

        let b_mono = QMonomial::new(QRat::from((2i64, 1i64)), 1); // b = 2*q
        let bv = compute_winquist_one_symbolic("z", &b_mono, sym_q, trunc);
        assert!(!bv.is_zero(), "bivariate winquist(z, 2*q, q, 30) should be nonzero");

        // Evaluate at z = -1: z^n -> (-1)^n (no q-exponent shift)
        let mut evaluated = FormalPowerSeries::zero(sym_q, trunc);
        for (&z_exp, fps) in &bv.terms {
            let sign = if z_exp % 2 == 0 { QRat::one() } else { -QRat::one() };
            for (&q_pow, coeff) in fps.iter() {
                if q_pow >= 0 && q_pow < trunc {
                    let old = evaluated.coeff(q_pow);
                    evaluated.set_coeff(q_pow, old + sign.clone() * coeff.clone());
                }
            }
        }

        // Compute numeric winquist(-1, 2*q, q, 30) via product form
        let a_mono = QMonomial::new(-QRat::one(), 0); // a = -1 (constant)
        let numeric = qseries::winquist(&a_mono, &b_mono, sym_q, trunc);
        assert!(!numeric.is_zero(), "numeric winquist(-1, 2*q, q, 30) should be nonzero");

        let safe_bound = trunc / 3;
        for k in 0..safe_bound {
            assert_eq!(
                evaluated.coeff(k), numeric.coeff(k),
                "winquist bivariate at z=-1, q^{} mismatch: got {:?} expected {:?}",
                k, evaluated.coeff(k), numeric.coeff(k)
            );
        }
    }

    #[test]
    fn dispatch_winquist_two_symbolic() {
        let mut env = make_env();
        // winquist(a, b, q, 5) with both a and b symbolic
        let args = vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let result = dispatch("winquist", &args, &mut env).unwrap();
        if let Value::TrivariateSeries(ts) = &result {
            assert!(!ts.is_zero(), "trivariate winquist should have nonzero terms");
            assert!(ts.terms.len() > 1, "trivariate winquist should have multiple (a, b) terms");
            // The (0, 0) term should exist (constant in a, b)
            assert!(ts.terms.contains_key(&(0, 0)),
                "trivariate winquist should have a (0, 0) term");
        } else {
            panic!("expected TrivariateSeries, got {:?}", result.type_name());
        }
    }

    #[test]
    fn winquist_two_symbolic_cross_validation() {
        // CRITICAL correctness test: trivariate winquist evaluated at a=-1, b=-1
        // must match numeric winquist(-1, -1, q, T).
        //
        // Using a=-1, b=-1 (constant q-monomials with power 0) avoids:
        //   1. Product zeros (no factor becomes (1-1))
        //   2. Truncation boundary effects (no q-shift from substitution)
        // At a=-1, b=-1: contribution from ((ra, rb), fps) is (-1)^ra * (-1)^rb * fps,
        // which simply sums all fps with sign (-1)^{ra+rb}. No q-shifting involved.
        let env = make_env();
        let sym_q = env.sym_q;
        let trunc: i64 = 10;

        // Compute trivariate
        let ts = compute_winquist_two_symbolic("a", "b", sym_q, trunc);

        // Evaluate at a = -1, b = -1
        let mut evaluated = FormalPowerSeries::zero(sym_q, trunc);
        for (&(ra, rb), fps) in &ts.terms {
            let sign = if (ra + rb) % 2 == 0 { QRat::one() } else { -QRat::one() };
            for (&p, v) in fps.iter() {
                if p < trunc {
                    let old = evaluated.coeff(p);
                    evaluated.set_coeff(p, old + sign.clone() * v.clone());
                }
            }
        }

        // Compute numeric reference: winquist(-1, -1, q, T)
        let a_mono = QMonomial::new(-QRat::one(), 0);
        let b_mono = QMonomial::new(-QRat::one(), 0);
        let reference = qseries::winquist(&a_mono, &b_mono, sym_q, trunc);

        // Compare coefficients
        for k in 0..trunc {
            assert_eq!(
                evaluated.coeff(k), reference.coeff(k),
                "Mismatch at q^{}: trivariate eval = {}, numeric = {}",
                k, evaluated.coeff(k), reference.coeff(k)
            );
        }

        // SECOND cross-validation: a=2, b=3 (positive constants, no q-shift)
        let mut evaluated2 = FormalPowerSeries::zero(sym_q, trunc);
        let two = QRat::from((2i64, 1i64));
        let three = QRat::from((3i64, 1i64));
        for (&(ra, rb), fps) in &ts.terms {
            let scalar = qrat_pow(&two, ra) * qrat_pow(&three, rb);
            for (&p, v) in fps.iter() {
                if p < trunc {
                    let old = evaluated2.coeff(p);
                    evaluated2.set_coeff(p, old + scalar.clone() * v.clone());
                }
            }
        }
        let a2_mono = QMonomial::new(two.clone(), 0);
        let b2_mono = QMonomial::new(three.clone(), 0);
        let reference2 = qseries::winquist(&a2_mono, &b2_mono, sym_q, trunc);
        for k in 0..trunc {
            assert_eq!(
                evaluated2.coeff(k), reference2.coeff(k),
                "2nd validation mismatch at q^{}: trivariate eval = {}, numeric = {}",
                k, evaluated2.coeff(k), reference2.coeff(k)
            );
        }
    }

    #[test]
    fn winquist_two_symbolic_display_not_empty() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        let formatted = crate::format::format_value(&val, &env.symbols);
        assert!(formatted.contains("a"), "display should contain 'a': {}", formatted);
        assert!(formatted.contains("b"), "display should contain 'b': {}", formatted);
        assert!(formatted.contains("q"), "display should contain 'q': {}", formatted);
        assert!(formatted.len() > 10, "display should not be trivially short: {}", formatted);
    }

    #[test]
    fn winquist_preserves_one_symbolic() {
        let mut env = make_env();
        // winquist(z, q^2, q, 5) should still return BivariateSeries
        let args = vec![
            Value::Symbol("z".to_string()),
            make_monomial_series(&env, 2),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        assert!(matches!(val, Value::BivariateSeries(_)),
            "winquist with one symbolic should return BivariateSeries, got {:?}", val.type_name());
    }

    #[test]
    fn winquist_preserves_numeric() {
        let mut env = make_env();
        // winquist(q, q^2, q, 10) where both args are q-monomials
        let args = vec![
            make_monomial_series(&env, 1),
            make_monomial_series(&env, 2),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("winquist", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)),
            "winquist with concrete monomials should return Series, got {:?}", val.type_name());
    }

    // --- Phase 48 FIX-01/FIX-02 Tests ---

    #[test]
    fn dispatch_aqprod_maple_3arg_polynomial_order() {
        let mut env = make_env();
        // aqprod(q, q, 5) -- should produce exact polynomial with POLYNOMIAL_ORDER
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("aqprod", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.truncation_order(), POLYNOMIAL_ORDER,
                "3-arg aqprod should use POLYNOMIAL_ORDER sentinel");
            // (q;q)_5 = (1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)
            //         = 1 - q - q^2 + q^5 + q^6 + q^7 - q^8 - q^9 - q^10 + q^13 + q^14 - q^15
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((-1i64, 1i64)));
            assert_eq!(fps.coeff(2), QRat::from((-1i64, 1i64)));
            assert_eq!(fps.coeff(3), QRat::zero());
            assert_eq!(fps.coeff(5), QRat::one());
            assert_eq!(fps.coeff(6), QRat::one());
            assert_eq!(fps.coeff(7), QRat::one());
            assert_eq!(fps.coeff(8), QRat::from((-1i64, 1i64)));
            assert_eq!(fps.coeff(9), QRat::from((-1i64, 1i64)));
            assert_eq!(fps.coeff(15), QRat::from((-1i64, 1i64)));
        } else {
            panic!("expected Series, got {:?}", val.type_name());
        }
    }

    #[test]
    fn dispatch_aqprod_4arg_unchanged() {
        let mut env = make_env();
        // aqprod(q, q, 5, 10) -- 4-arg form should use explicit truncation order
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("aqprod", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.truncation_order(), 10,
                "4-arg aqprod should use explicit truncation order");
        } else {
            panic!("expected Series, got {:?}", val.type_name());
        }
    }

    #[test]
    fn dispatch_theta3_2arg() {
        let mut env = make_env();
        // theta3(q, 20) should match theta3(20)
        let args_1arg = vec![Value::Integer(QInt::from(20i64))];
        let val_1arg = dispatch("theta3", &args_1arg, &mut env).unwrap();

        let args_2arg = vec![
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(20i64)),
        ];
        let val_2arg = dispatch("theta3", &args_2arg, &mut env).unwrap();

        if let (Value::Series(fps1), Value::Series(fps2)) = (&val_1arg, &val_2arg) {
            assert_eq!(fps1.coeff(0), fps2.coeff(0));
            assert_eq!(fps1.coeff(1), fps2.coeff(1));
            assert_eq!(fps1.coeff(4), fps2.coeff(4));
            assert_eq!(fps1.coeff(9), fps2.coeff(9));
        } else {
            panic!("expected both to be Series");
        }
    }

    #[test]
    fn dispatch_theta3_3arg() {
        let mut env = make_env();
        // theta3(q, q, 100) should match theta3(100) -- roadmap success criterion
        let args_1arg = vec![Value::Integer(QInt::from(100i64))];
        let val_1arg = dispatch("theta3", &args_1arg, &mut env).unwrap();

        let args_3arg = vec![
            Value::Symbol("q".to_string()),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(100i64)),
        ];
        let val_3arg = dispatch("theta3", &args_3arg, &mut env).unwrap();

        if let (Value::Series(fps1), Value::Series(fps3)) = (&val_1arg, &val_3arg) {
            assert_eq!(fps1.coeff(0), fps3.coeff(0));
            assert_eq!(fps1.coeff(1), fps3.coeff(1));
            assert_eq!(fps1.coeff(4), fps3.coeff(4));
            assert_eq!(fps1.coeff(9), fps3.coeff(9));
            assert_eq!(fps1.coeff(16), fps3.coeff(16));
        } else {
            panic!("expected both to be Series");
        }
    }

    #[test]
    fn dispatch_theta2_2arg() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("theta2", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)),
            "theta2(q, 10) should return Series");
    }

    #[test]
    fn dispatch_theta4_2arg() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(10i64)),
        ];
        let val = dispatch("theta4", &args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)),
            "theta4(q, 10) should return Series");
    }

    #[test]
    fn dispatch_theta3_1arg_unchanged() {
        let mut env = make_env();
        // Regression test: theta3(20) still works
        let args = vec![Value::Integer(QInt::from(20i64))];
        let val = dispatch("theta3", &args, &mut env).unwrap();
        if let Value::Series(fps) = val {
            // theta3 = 1 + 2q + 2q^4 + 2q^9 + 2q^16 + ...
            assert_eq!(fps.coeff(0), QRat::one());
            assert_eq!(fps.coeff(1), QRat::from((2i64, 1i64)));
            assert_eq!(fps.coeff(4), QRat::from((2i64, 1i64)));
        } else {
            panic!("expected Series");
        }
    }

    // --- qfactor 2-arg Integer detection (FIX-05) ---

    #[test]
    fn dispatch_qfactor_2arg_integer() {
        let mut env = make_env();
        // Create a series via qbin(5,2,20)
        let qb = dispatch("qbin", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        // qfactor(f, 100) with Integer second arg
        let val = dispatch("qfactor", &[
            qb,
            Value::Integer(QInt::from(100i64)),
        ], &mut env).unwrap();
        assert!(matches!(val, Value::QProduct { .. }), "expected QProduct from qfactor(f, Integer), got {:?}", val);
    }

    #[test]
    fn dispatch_qfactor_2arg_symbol_still_works() {
        let mut env = make_env();
        let qb = dispatch("qbin", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        // qfactor(f, q) with Symbol second arg (regression test)
        let val = dispatch("qfactor", &[
            qb,
            Value::Symbol("q".to_string()),
        ], &mut env).unwrap();
        assert!(matches!(val, Value::QProduct { .. }), "expected QProduct from qfactor(f, Symbol), got {:?}", val);
    }

    // --- min/max variadic functions (LANG-03) ---

    #[test]
    fn dispatch_min_integers() {
        let mut env = make_env();
        let val = dispatch("min", &[
            Value::Integer(QInt::from(3i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(4i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = &val {
            assert_eq!(*n, QInt::from(1i64), "min(3,1,4,1,5) should be 1");
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_min_rationals() {
        let mut env = make_env();
        let val = dispatch("min", &[
            Value::Rational(QRat::from((1i64, 3i64))),
            Value::Rational(QRat::from((1i64, 2i64))),
        ], &mut env).unwrap();
        if let Value::Rational(r) = &val {
            assert_eq!(*r, QRat::from((1i64, 3i64)), "min(1/3, 1/2) should be 1/3");
        } else {
            panic!("expected Rational, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_min_mixed() {
        let mut env = make_env();
        let val = dispatch("min", &[
            Value::Integer(QInt::from(2i64)),
            Value::Rational(QRat::from((3i64, 2i64))),
        ], &mut env).unwrap();
        if let Value::Rational(r) = &val {
            assert_eq!(*r, QRat::from((3i64, 2i64)), "min(2, 3/2) should be 3/2");
        } else {
            panic!("expected Rational, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_min_single() {
        let mut env = make_env();
        let val = dispatch("min", &[
            Value::Integer(QInt::from(7i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = &val {
            assert_eq!(*n, QInt::from(7i64), "min(7) should be 7");
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_min_empty_error() {
        let mut env = make_env();
        let result = dispatch("min", &[], &mut env);
        assert!(result.is_err(), "min() with no args should error");
        if let Err(EvalError::WrongArgCount { function, .. }) = result {
            assert_eq!(function, "min");
        } else {
            panic!("expected WrongArgCount error");
        }
    }

    #[test]
    fn dispatch_max_integers() {
        let mut env = make_env();
        let val = dispatch("max", &[
            Value::Integer(QInt::from(3i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
        ], &mut env).unwrap();
        if let Value::Integer(n) = &val {
            assert_eq!(*n, QInt::from(5i64), "max(3,1,5) should be 5");
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_max_rationals() {
        let mut env = make_env();
        let val = dispatch("max", &[
            Value::Rational(QRat::from((1i64, 3i64))),
            Value::Rational(QRat::from((1i64, 2i64))),
        ], &mut env).unwrap();
        if let Value::Rational(r) = &val {
            assert_eq!(*r, QRat::from((1i64, 2i64)), "max(1/3, 1/2) should be 1/2");
        } else {
            panic!("expected Rational, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_min_preserves_integer_type() {
        let mut env = make_env();
        let val = dispatch("min", &[
            Value::Integer(QInt::from(3i64)),
            Value::Integer(QInt::from(1i64)),
        ], &mut env).unwrap();
        // Must return Integer, not Rational (avoids "1/1" display)
        assert!(matches!(val, Value::Integer(_)),
            "min of integers should return Integer, got {:?}", val);
    }

    #[test]
    fn integration_etamake_displays_eta_notation() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("etamake(partition_gf(50), q, 10)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert!(text.contains("eta(tau)"), "expected eta(tau) in: {}", text);
        assert!(!text.contains("factors"), "should not show raw dict: {}", text);
    }

    #[test]
    fn subs_multi_substitution() {
        // subs(q=1, 1 + q + q^2) should still work with single pair (backward compat)
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("subs(q=0, 1 + q + q^2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        // subs(q=0, 1+q+q^2) should give constant term 1
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(1i64), "subs(q=0, 1+q+q^2) should be 1");
        } else if let Value::Rational(r) = &result {
            assert_eq!(*r, QRat::from((1i64, 1i64)), "subs(q=0, 1+q+q^2) should be 1");
        } else {
            panic!("expected Integer or Rational, got {:?}", result);
        }
    }

    #[test]
    fn subs_indexed_variable() {
        // Test that indexed variable names work -- X[1] is just a string name
        use crate::parser::parse;
        let mut env = make_env();
        // Assign X[1] then substitute
        let stmts = parse("X[1] := 42; X[1]").unwrap();
        for stmt in &stmts {
            let _ = eval_stmt(stmt, &mut env);
        }
        let result = env.get_var("X[1]").cloned();
        assert!(result.is_some(), "X[1] should be defined");
        if let Some(Value::Integer(n)) = result {
            assert_eq!(n, QInt::from(42i64));
        }
    }

    #[test]
    fn dispatch_theta3_monomial() {
        // theta3(q^2, 10) should produce theta3 with exponents scaled by 2
        // theta3(q) = 1 + 2q + 2q^4 + 2q^9 + ...
        // theta3(q^2, 10) = 1 + 2q^2 + 2q^8 + 2q^18 + ... up to O(q^20)
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        // Build q^2 as a monomial series (POLYNOMIAL_ORDER sentinel)
        let mono = FormalPowerSeries::monomial(sym, QRat::one(), 2, 1000000000);
        let args = vec![Value::Series(mono), Value::Integer(QInt::from(10i64))];
        let result = dispatch("theta3", &args, &mut env).unwrap();
        if let Value::Series(fps) = &result {
            // Should have constant term 1
            assert_eq!(fps.coeff(0), QRat::one(), "theta3(q^2,10) constant term should be 1");
            // Should have coeff 2 at q^2 (n=1: 1^2*2 = 2)
            assert_eq!(fps.coeff(2), QRat::from((2i64, 1i64)), "theta3(q^2,10) should have 2*q^2");
            // Should have zero at q^1 (no odd exponents)
            assert_eq!(fps.coeff(1), QRat::zero(), "theta3(q^2,10) should have no q^1 term");
            // Should have coeff 2 at q^8 (n=2: 4*2 = 8)
            assert_eq!(fps.coeff(8), QRat::from((2i64, 1i64)), "theta3(q^2,10) should have 2*q^8");
            // Should have coeff 2 at q^18 (n=3: 9*2 = 18)
            assert_eq!(fps.coeff(18), QRat::from((2i64, 1i64)), "theta3(q^2,10) should have 2*q^18");
            // Truncation should be 20 (10 * 2)
            assert_eq!(fps.truncation_order(), 20, "truncation_order should be 20");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_radsimp_identity() {
        // radsimp(series) should return the series unchanged
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        let fps = qseries::theta3(sym, 10);
        let args = vec![Value::Series(fps.clone())];
        let result = dispatch("radsimp", &args, &mut env).unwrap();
        if let Value::Series(result_fps) = &result {
            assert_eq!(*result_fps, fps, "radsimp should return series unchanged");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_radsimp_integer() {
        // radsimp(5) should return 5
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64))];
        let result = dispatch("radsimp", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(5i64), "radsimp(5) should return 5");
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    // =======================================================
    // POLYNOMIAL_ORDER division cap tests (BUG-01)
    // =======================================================

    #[test]
    fn cap_poly_order_with_polynomial_order() {
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        // Build a series with POLYNOMIAL_ORDER truncation
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0, QRat::one());
        coeffs.insert(1, QRat::from((-1i64, 1i64)));
        let fps = FormalPowerSeries::from_coeffs(sym, coeffs, POLYNOMIAL_ORDER);
        assert_eq!(fps.truncation_order(), POLYNOMIAL_ORDER);
        // Cap to 10
        let capped = cap_poly_order(&fps, 10);
        assert_eq!(capped.truncation_order(), 10, "should cap to fallback");
        assert_eq!(capped.coeff(0), QRat::one(), "constant term preserved");
        assert_eq!(capped.coeff(1), QRat::from((-1i64, 1i64)), "q^1 preserved");
    }

    #[test]
    fn cap_poly_order_normal_passthrough() {
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        let fps = FormalPowerSeries::monomial(sym, QRat::one(), 0, 20);
        let capped = cap_poly_order(&fps, 10);
        // Should NOT change truncation since it's not POLYNOMIAL_ORDER
        assert_eq!(capped.truncation_order(), 20, "normal series unchanged");
    }

    #[test]
    fn div_scalar_by_polynomial_order_series() {
        // Test 1/(series with POLYNOMIAL_ORDER) completes -- the key BUG-01 regression test.
        // Before the fix, this would hang trying to loop 1 billion times in invert().
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        // Build a series with POLYNOMIAL_ORDER truncation (simulates aqprod 3-arg output)
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0, QRat::one());
        coeffs.insert(1, QRat::from((-1i64, 1i64)));
        coeffs.insert(2, QRat::from((-1i64, 1i64)));
        let poly_series = FormalPowerSeries::from_coeffs(sym, coeffs, POLYNOMIAL_ORDER);
        assert_eq!(poly_series.truncation_order(), POLYNOMIAL_ORDER);
        // Now do 1 / poly_series -- this should NOT hang
        let result = eval_div(
            Value::Integer(QInt::from(1i64)),
            Value::Series(poly_series),
            &mut env,
        ).unwrap();
        if let Value::Series(fps) = result {
            // Should complete and have a constant term
            assert_eq!(fps.coeff(0), QRat::one(), "1/(poly_order series) should have constant term 1");
            // Truncation should be env.default_order (20), not POLYNOMIAL_ORDER
            assert!(fps.truncation_order() <= env.default_order,
                "truncation should be capped to default_order");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn div_series_series_both_polynomial_order() {
        // Both sides POLYNOMIAL_ORDER -> use env.default_order
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        let mut coeffs_a = BTreeMap::new();
        coeffs_a.insert(0, QRat::one());
        coeffs_a.insert(1, QRat::from((2i64, 1i64)));
        let a = FormalPowerSeries::from_coeffs(sym, coeffs_a, POLYNOMIAL_ORDER);
        let mut coeffs_b = BTreeMap::new();
        coeffs_b.insert(0, QRat::one());
        let b = FormalPowerSeries::from_coeffs(sym, coeffs_b, POLYNOMIAL_ORDER);
        let result = eval_div(
            Value::Series(a),
            Value::Series(b),
            &mut env,
        ).unwrap();
        if let Value::Series(fps) = result {
            assert!(fps.truncation_order() <= env.default_order,
                "both POLYNOMIAL_ORDER should cap to default_order");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    #[test]
    fn div_normal_series_still_works() {
        // Normal division (non-POLYNOMIAL_ORDER) should still work correctly
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        let mut coeffs_a = BTreeMap::new();
        coeffs_a.insert(0, QRat::one());
        let a = FormalPowerSeries::from_coeffs(sym, coeffs_a, 10);
        let mut coeffs_b = BTreeMap::new();
        coeffs_b.insert(0, QRat::from((2i64, 1i64)));
        let b = FormalPowerSeries::from_coeffs(sym, coeffs_b, 10);
        let result = eval_div(
            Value::Series(a),
            Value::Series(b),
            &mut env,
        ).unwrap();
        if let Value::Series(fps) = result {
            assert_eq!(fps.coeff(0), QRat::from((1i64, 2i64)), "1/2 constant term");
        } else {
            panic!("expected Series, got {:?}", result);
        }
    }

    // -- While-loop eval tests ------------------------------------------------

    #[test]
    fn test_while_loop_basic() {
        // i:=0: while i<10 do i:=i+1 od: i  =>  10
        let mut env = make_env();
        env.set_var("i", Value::Integer(QInt::from(0i64)));
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Variable("i".to_string())),
                rhs: Box::new(AstNode::Integer(10)),
            }),
            body: vec![Stmt {
                node: AstNode::Assign {
                    name: "i".to_string(),
                    value: Box::new(AstNode::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(AstNode::Variable("i".to_string())),
                        rhs: Box::new(AstNode::Integer(1)),
                    }),
                },
                terminator: Terminator::Colon,
            }],
        };
        let _result = eval_expr(&node, &mut env).unwrap();
        // Check i is now 10
        if let Some(Value::Integer(n)) = env.get_var("i") {
            assert_eq!(*n, QInt::from(10i64));
        } else {
            panic!("i should be Integer(10)");
        }
    }

    #[test]
    fn test_while_loop_doubling() {
        // x:=1: while x<100 do x:=x*2 od: x  =>  128
        let mut env = make_env();
        env.set_var("x", Value::Integer(QInt::from(1i64)));
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(100)),
            }),
            body: vec![Stmt {
                node: AstNode::Assign {
                    name: "x".to_string(),
                    value: Box::new(AstNode::BinOp {
                        op: BinOp::Mul,
                        lhs: Box::new(AstNode::Variable("x".to_string())),
                        rhs: Box::new(AstNode::Integer(2)),
                    }),
                },
                terminator: Terminator::Colon,
            }],
        };
        let _result = eval_expr(&node, &mut env).unwrap();
        if let Some(Value::Integer(n)) = env.get_var("x") {
            assert_eq!(*n, QInt::from(128i64));
        } else {
            panic!("x should be Integer(128)");
        }
    }

    #[test]
    fn test_while_loop_zero_iterations() {
        // while false do 42 od  =>  Value::None
        let mut env = make_env();
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Variable("false".to_string())),
            body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
        };
        // "false" resolves to Value::Bool(false) because the variable is unset
        // Actually, "false" as a Variable will fail because it's not defined.
        // Let's use a Compare that's immediately false.
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Greater,
                lhs: Box::new(AstNode::Integer(0)),
                rhs: Box::new(AstNode::Integer(1)),
            }),
            body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
        };
        let result = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(result, Value::None), "while with false condition should return None");
    }

    #[test]
    fn test_while_loop_safety_limit() {
        // while true do 1 od  => error "maximum iteration count"
        let mut env = make_env();
        // Use an integer 1 as condition (truthy via Maple convention: nonzero = true)
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Integer(1)),
            body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Colon,
            }],
        };
        let result = eval_expr(&node, &mut env);
        assert!(result.is_err(), "infinite while loop should hit safety limit");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("maximum iteration count"),
            "error should mention maximum iteration count: {}", err_msg);
    }

    #[test]
    fn while_symbol_true_hits_safety_limit() {
        // "true" is Variable("true") -> Symbol("true") -- is_truthy must accept it
        let mut env = make_env();
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Variable("true".to_string())),
            body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Colon,
            }],
        };
        let result = eval_expr(&node, &mut env);
        assert!(result.is_err(), "while true should hit safety limit");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("maximum iteration count"),
            "should hit safety limit, not type error: {}", err_msg);
    }

    #[test]
    fn while_symbol_false_does_not_execute() {
        // "false" is Variable("false") -> Symbol("false") -- is_truthy returns false
        let mut env = make_env();
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Variable("false".to_string())),
            body: vec![Stmt {
                node: AstNode::Integer(42),
                terminator: Terminator::Implicit,
            }],
        };
        let result = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(result, Value::None),
            "while false should return None, got: {:?}", result);
    }

    #[test]
    fn is_truthy_rejects_unknown_symbol() {
        // Unknown symbols like "x" should error, not be treated as true/false
        let mut env = make_env();
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Variable("x".to_string())),
            body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Implicit,
            }],
        };
        let result = eval_expr(&node, &mut env);
        assert!(result.is_err(), "unknown symbol in condition should error");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("symbol 'x'"),
            "error should mention the symbol name: {}", err_msg);
    }

    #[test]
    fn test_while_loop_comparison_operators() {
        // Test while with each comparison operator
        let ops_and_setups: Vec<(CompOp, i64, i64, i64)> = vec![
            // (op, initial_x, rhs_bound, expected_final_x)
            (CompOp::Less, 0, 5, 5),        // while x < 5
            (CompOp::LessEq, 0, 5, 6),      // while x <= 5
            (CompOp::Greater, 5, 0, 0),      // while x > 0
            (CompOp::GreaterEq, 5, 0, -1),   // while x >= 0
            (CompOp::NotEq, 0, 3, 3),        // while x <> 3
        ];
        for (op, init, bound, expected) in ops_and_setups {
            let mut env = make_env();
            env.set_var("x", Value::Integer(QInt::from(init)));
            let step = if init <= bound { 1 } else { -1 };
            let node = AstNode::WhileLoop {
                condition: Box::new(AstNode::Compare {
                    op,
                    lhs: Box::new(AstNode::Variable("x".to_string())),
                    rhs: Box::new(AstNode::Integer(bound)),
                }),
                body: vec![Stmt {
                    node: AstNode::Assign {
                        name: "x".to_string(),
                        value: Box::new(AstNode::BinOp {
                            op: BinOp::Add,
                            lhs: Box::new(AstNode::Variable("x".to_string())),
                            rhs: Box::new(AstNode::Integer(step)),
                        }),
                    },
                    terminator: Terminator::Colon,
                }],
            };
            let _result = eval_expr(&node, &mut env).unwrap();
            if let Some(Value::Integer(n)) = env.get_var("x") {
                assert_eq!(*n, QInt::from(expected),
                    "while x {:?} {} with step {} should end at {}", op, bound, step, expected);
            } else {
                panic!("x should be Integer({})", expected);
            }
        }
    }

    #[test]
    fn test_while_loop_with_eq() {
        // while x = 0 do x := 1 od  =>  x = 1 (one iteration)
        let mut env = make_env();
        env.set_var("x", Value::Integer(QInt::from(0i64)));
        let node = AstNode::WhileLoop {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Eq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(0)),
            }),
            body: vec![Stmt {
                node: AstNode::Assign {
                    name: "x".to_string(),
                    value: Box::new(AstNode::Integer(1)),
                },
                terminator: Terminator::Colon,
            }],
        };
        let _result = eval_expr(&node, &mut env).unwrap();
        if let Some(Value::Integer(n)) = env.get_var("x") {
            assert_eq!(*n, QInt::from(1i64));
        } else {
            panic!("x should be Integer(1)");
        }
    }

    #[test]
    fn test_while_nested_in_for() {
        // for n from 1 to 3 do s := s + n od with while-loop inside
        // s:=0: for n from 1 to 3 do x:=0: while x < n do x:=x+1: s:=s+1 od od: s
        // should be s = 1+2+3 = 6
        let mut env = make_env();
        env.set_var("s", Value::Integer(QInt::from(0i64)));
        let while_body = vec![
            Stmt {
                node: AstNode::Assign {
                    name: "x".to_string(),
                    value: Box::new(AstNode::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(AstNode::Variable("x".to_string())),
                        rhs: Box::new(AstNode::Integer(1)),
                    }),
                },
                terminator: Terminator::Colon,
            },
            Stmt {
                node: AstNode::Assign {
                    name: "s".to_string(),
                    value: Box::new(AstNode::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(AstNode::Variable("s".to_string())),
                        rhs: Box::new(AstNode::Integer(1)),
                    }),
                },
                terminator: Terminator::Colon,
            },
        ];
        let for_body = vec![
            Stmt {
                node: AstNode::Assign {
                    name: "x".to_string(),
                    value: Box::new(AstNode::Integer(0)),
                },
                terminator: Terminator::Colon,
            },
            Stmt {
                node: AstNode::WhileLoop {
                    condition: Box::new(AstNode::Compare {
                        op: CompOp::Less,
                        lhs: Box::new(AstNode::Variable("x".to_string())),
                        rhs: Box::new(AstNode::Variable("n".to_string())),
                    }),
                    body: while_body,
                },
                terminator: Terminator::Colon,
            },
        ];
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(3)),
            by: None,
            body: for_body,
        };
        let _result = eval_expr(&node, &mut env).unwrap();
        if let Some(Value::Integer(n)) = env.get_var("s") {
            assert_eq!(*n, QInt::from(6i64), "s should be 6 (1+2+3)");
        } else {
            panic!("s should be Integer(6)");
        }
    }

    // =======================================================
    // print() special-case function tests (LANG-04)
    // =======================================================

    #[test]
    fn print_zero_args_errors() {
        let mut env = make_env();
        let node = AstNode::FuncCall {
            name: "print".to_string(),
            args: vec![],
        };
        let result = eval_expr(&node, &mut env);
        assert!(result.is_err(), "print() with no args should error");
        let err = result.unwrap_err();
        if let EvalError::WrongArgCount { function, .. } = err {
            assert_eq!(function, "print");
        } else {
            panic!("expected WrongArgCount, got {:?}", err);
        }
    }

    #[test]
    fn print_single_integer_returns_value() {
        // print(42) should return Value::Integer(42)
        let mut env = make_env();
        let node = AstNode::FuncCall {
            name: "print".to_string(),
            args: vec![AstNode::Integer(42)],
        };
        let result = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(42i64));
        } else {
            panic!("expected Integer(42), got {:?}", result);
        }
    }

    #[test]
    fn print_multiple_args_returns_last() {
        // print(1, 2, 3) should return Value::Integer(3) (the last arg)
        let mut env = make_env();
        let node = AstNode::FuncCall {
            name: "print".to_string(),
            args: vec![
                AstNode::Integer(1),
                AstNode::Integer(2),
                AstNode::Integer(3),
            ],
        };
        let result = eval_expr(&node, &mut env).unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, QInt::from(3i64), "print returns last arg value");
        } else {
            panic!("expected Integer(3), got {:?}", result);
        }
    }

    #[test]
    fn print_with_series_returns_series() {
        // print(series) should return the series value
        let mut env = make_env();
        let sym = env.symbols.intern("q");
        let fps = qseries::theta3(sym, 10);
        env.set_var("f", Value::Series(fps));
        let node = AstNode::FuncCall {
            name: "print".to_string(),
            args: vec![AstNode::Variable("f".to_string())],
        };
        let result = eval_expr(&node, &mut env).unwrap();
        assert!(matches!(result, Value::Series(_)), "print should return Series");
    }

    // --- List indexing tests ---

    #[test]
    fn eval_list_indexing() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("L := [10, 20, 30]; L[1]").unwrap();
        let mut last = Value::None;
        for stmt in &stmts {
            if let Some(val) = eval_stmt(stmt, &mut env).unwrap() {
                last = val;
            }
        }
        if let Value::Integer(n) = &last {
            assert_eq!(*n, QInt::from(10i64), "L[1] should be 10");
        } else {
            panic!("expected Integer, got {:?}", last);
        }
        // Also check L[2] and L[3]
        let stmts2 = parse("L[2]").unwrap();
        let val2 = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = &val2 {
            assert_eq!(*n, QInt::from(20i64), "L[2] should be 20");
        } else {
            panic!("expected Integer for L[2], got {:?}", val2);
        }
        let stmts3 = parse("L[3]").unwrap();
        let val3 = eval_stmt(&stmts3[0], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = &val3 {
            assert_eq!(*n, QInt::from(30i64), "L[3] should be 30");
        } else {
            panic!("expected Integer for L[3], got {:?}", val3);
        }
    }

    #[test]
    fn eval_list_index_out_of_range() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("L := [1, 2]; L[3]").unwrap();
        // First statement assigns, second should error
        eval_stmt(&stmts[0], &mut env).unwrap();
        let result = eval_stmt(&stmts[1], &mut env);
        assert!(result.is_err(), "L[3] on 2-element list should error");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"), "error should mention out of range: {}", err);
    }

    #[test]
    fn eval_list_index_zero() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("L := [1, 2]; L[0]").unwrap();
        eval_stmt(&stmts[0], &mut env).unwrap();
        let result = eval_stmt(&stmts[1], &mut env);
        assert!(result.is_err(), "L[0] should error (1-indexed)");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"), "error should mention out of range: {}", err);
    }

    #[test]
    fn eval_list_index_assign() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("L := [1, 2, 3]; L[2] := 99; L[2]").unwrap();
        for stmt in &stmts[..2] {
            eval_stmt(stmt, &mut env).unwrap();
        }
        let result = eval_stmt(&stmts[2], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(99i64), "L[2] should be 99 after assignment");
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn eval_table_style_backward_compat() {
        use crate::parser::parse;
        let mut env = make_env();
        // X is not defined as a list -- falls back to table-style
        let stmts = parse("X[1] := 42; X[1]").unwrap();
        for stmt in &stmts[..1] {
            eval_stmt(stmt, &mut env).unwrap();
        }
        let result = eval_stmt(&stmts[1], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(42i64), "X[1] should be 42 (table-style)");
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn eval_list_literal_index() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("[10, 20, 30][2]").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(20i64), "[10,20,30][2] should be 20");
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // nops dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_nops_list() {
        let mut env = make_env();
        let args = vec![Value::List(vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(3i64)),
        ])];
        let result = dispatch("nops", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(3i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_nops_integer() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(42i64))];
        let result = dispatch("nops", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(1i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_nops_series() {
        let mut env = make_env();
        // Create a simple series with 3 nonzero terms: 1 + q + q^2 + O(q^5)
        let sym = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(0, QRat::one());
        coeffs.insert(1, QRat::one());
        coeffs.insert(2, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym, coeffs, 5);
        let args = vec![Value::Series(fps)];
        let result = dispatch("nops", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(3i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // op dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_op_list() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(2i64)),
            Value::List(vec![
                Value::Integer(QInt::from(10i64)),
                Value::Integer(QInt::from(20i64)),
                Value::Integer(QInt::from(30i64)),
            ]),
        ];
        let result = dispatch("op", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(20i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_op_out_of_range() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(4i64)),
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(2i64)),
                Value::Integer(QInt::from(3i64)),
            ]),
        ];
        let result = dispatch("op", &args, &mut env);
        assert!(result.is_err());
    }

    #[test]
    fn dispatch_op_series() {
        let mut env = make_env();
        // Create series: 2*q + 3*q^2 + O(q^5)
        let sym = env.sym_q;
        let mut coeffs = BTreeMap::new();
        coeffs.insert(1, QRat::from((2, 1)));
        coeffs.insert(2, QRat::from((3, 1)));
        let fps = FormalPowerSeries::from_coeffs(sym, coeffs, 5);
        let args = vec![
            Value::Integer(QInt::from(1i64)),
            Value::Series(fps),
        ];
        let result = dispatch("op", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 2);
            if let Value::Integer(exp) = &items[0] {
                assert_eq!(*exp, QInt::from(1i64), "first nonzero term exponent should be 1");
            } else {
                panic!("expected Integer exponent, got {:?}", items[0]);
            }
            if let Value::Integer(coeff) = &items[1] {
                assert_eq!(*coeff, QInt::from(2i64), "first nonzero term coefficient should be 2");
            } else {
                panic!("expected Integer coefficient, got {:?}", items[1]);
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // map dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_map_builtin() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("numbpart".to_string()),
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(2i64)),
                Value::Integer(QInt::from(3i64)),
                Value::Integer(QInt::from(4i64)),
                Value::Integer(QInt::from(5i64)),
            ]),
        ];
        let result = dispatch("map", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 5);
            let expected = [1i64, 2, 3, 5, 7];
            for (i, exp) in expected.iter().enumerate() {
                if let Value::Integer(n) = &items[i] {
                    assert_eq!(*n, QInt::from(*exp), "numbpart({}) should be {}", i + 1, exp);
                } else {
                    panic!("expected Integer at index {}, got {:?}", i, items[i]);
                }
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_map_symbol() {
        let mut env = make_env();
        // map(floor, [3/2, 5/2, 7/2]) should return [1, 2, 3]
        let args = vec![
            Value::Symbol("floor".to_string()),
            Value::List(vec![
                Value::Rational(QRat::from((3, 2))),
                Value::Rational(QRat::from((5, 2))),
                Value::Rational(QRat::from((7, 2))),
            ]),
        ];
        let result = dispatch("map", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 3);
            let expected = [1i64, 2, 3];
            for (i, exp) in expected.iter().enumerate() {
                if let Value::Integer(n) = &items[i] {
                    assert_eq!(*n, QInt::from(*exp));
                } else {
                    panic!("expected Integer at index {}, got {:?}", i, items[i]);
                }
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // sort dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_sort_integers() {
        let mut env = make_env();
        let args = vec![Value::List(vec![
            Value::Integer(QInt::from(3i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
        ])];
        let result = dispatch("sort", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 3);
            let expected = [1i64, 2, 3];
            for (i, exp) in expected.iter().enumerate() {
                if let Value::Integer(n) = &items[i] {
                    assert_eq!(*n, QInt::from(*exp));
                } else {
                    panic!("expected Integer at index {}, got {:?}", i, items[i]);
                }
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_sort_rationals() {
        let mut env = make_env();
        let args = vec![Value::List(vec![
            Value::Rational(QRat::from((3, 2))),
            Value::Rational(QRat::from((1, 2))),
            Value::Rational(QRat::from((5, 2))),
        ])];
        let result = dispatch("sort", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 3);
            let expected = [(1, 2), (3, 2), (5, 2)];
            for (i, (n, d)) in expected.iter().enumerate() {
                if let Value::Rational(r) = &items[i] {
                    assert_eq!(*r, QRat::from((*n, *d)));
                } else {
                    panic!("expected Rational at index {}, got {:?}", i, items[i]);
                }
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_sort_mixed_numeric() {
        let mut env = make_env();
        let args = vec![Value::List(vec![
            Value::Integer(QInt::from(3i64)),
            Value::Rational(QRat::from((1, 2))),
            Value::Integer(QInt::from(2i64)),
        ])];
        let result = dispatch("sort", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 3);
            // Expected order: 1/2, 2, 3
            if let Value::Rational(r) = &items[0] {
                assert_eq!(*r, QRat::from((1, 2)));
            } else {
                panic!("expected Rational(1/2) at index 0, got {:?}", items[0]);
            }
            if let Value::Integer(n) = &items[1] {
                assert_eq!(*n, QInt::from(2i64));
            } else {
                panic!("expected Integer(2) at index 1, got {:?}", items[1]);
            }
            if let Value::Integer(n) = &items[2] {
                assert_eq!(*n, QInt::from(3i64));
            } else {
                panic!("expected Integer(3) at index 2, got {:?}", items[2]);
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_sort_symbols() {
        let mut env = make_env();
        let args = vec![Value::List(vec![
            Value::Symbol("c".to_string()),
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
        ])];
        let result = dispatch("sort", &args, &mut env).unwrap();
        if let Value::List(items) = &result {
            assert_eq!(items.len(), 3);
            let expected = ["a", "b", "c"];
            for (i, exp) in expected.iter().enumerate() {
                if let Value::Symbol(s) = &items[i] {
                    assert_eq!(s, exp);
                } else {
                    panic!("expected Symbol at index {}, got {:?}", i, items[i]);
                }
            }
        } else {
            panic!("expected List, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // Integration tests (parse + eval)
    // -----------------------------------------------------------------------

    #[test]
    fn eval_nops_list_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("nops([1,2,3])").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "3");
    }

    #[test]
    fn eval_op_list_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("op(2, [10, 20, 30])").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "20");
    }

    #[test]
    fn eval_map_with_lambda() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("map(x -> x*x, [1, 2, 3, 4])").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "[1, 4, 9, 16]");
    }

    #[test]
    fn eval_sort_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("sort([3, 1, 2])").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "[1, 2, 3]");
    }

    // -----------------------------------------------------------------------
    // coeff dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_coeff_integer_constant() {
        let mut env = make_env();
        // coeff(42, q, 0) = 42
        let args = vec![
            Value::Integer(QInt::from(42i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(0i64)),
        ];
        let result = dispatch("coeff", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(42i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }

        // coeff(42, q, 1) = 0
        let args = vec![
            Value::Integer(QInt::from(42i64)),
            Value::Symbol("q".to_string()),
            Value::Integer(QInt::from(1i64)),
        ];
        let result = dispatch("coeff", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(0i64));
        } else {
            panic!("expected Integer(0), got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // numer/denom dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_numer_rational() {
        let mut env = make_env();
        let args = vec![Value::Rational(QRat::from((3, 4)))];
        let result = dispatch("numer", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(3i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_denom_rational() {
        let mut env = make_env();
        let args = vec![Value::Rational(QRat::from((3, 4)))];
        let result = dispatch("denom", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(4i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_numer_integer() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(42i64))];
        let result = dispatch("numer", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(42i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_denom_integer() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(42i64))];
        let result = dispatch("denom", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(1i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // modp/mods dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_modp_basic() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(7i64)),
            Value::Integer(QInt::from(3i64)),
        ];
        let result = dispatch("modp", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(1i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_modp_negative() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(-7i64)),
            Value::Integer(QInt::from(3i64)),
        ];
        let result = dispatch("modp", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(2i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_mods_basic() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(7i64)),
            Value::Integer(QInt::from(3i64)),
        ];
        let result = dispatch("mods", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(1i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_mods_symmetric() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(3i64)),
        ];
        let result = dispatch("mods", &args, &mut env).unwrap();
        if let Value::Integer(n) = &result {
            assert_eq!(*n, QInt::from(-1i64));
        } else {
            panic!("expected Integer, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // type dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_type_integer() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(42i64)),
            Value::Symbol("integer".to_string()),
        ];
        let result = dispatch("type", &args, &mut env).unwrap();
        if let Value::Bool(b) = &result {
            assert!(*b);
        } else {
            panic!("expected Bool, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_type_series_false() {
        let mut env = make_env();
        let args = vec![
            Value::Integer(QInt::from(42i64)),
            Value::Symbol("series".to_string()),
        ];
        let result = dispatch("type", &args, &mut env).unwrap();
        if let Value::Bool(b) = &result {
            assert!(!*b);
        } else {
            panic!("expected Bool, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // evalb dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_evalb_true() {
        let mut env = make_env();
        let args = vec![Value::Bool(true)];
        let result = dispatch("evalb", &args, &mut env).unwrap();
        if let Value::Bool(b) = &result {
            assert!(*b);
        } else {
            panic!("expected Bool, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_evalb_zero() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(0i64))];
        let result = dispatch("evalb", &args, &mut env).unwrap();
        if let Value::Bool(b) = &result {
            assert!(!*b);
        } else {
            panic!("expected Bool, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_evalb_nonzero() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(42i64))];
        let result = dispatch("evalb", &args, &mut env).unwrap();
        if let Value::Bool(b) = &result {
            assert!(*b);
        } else {
            panic!("expected Bool, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // cat dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_cat_symbols() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("c".to_string()),
        ];
        let result = dispatch("cat", &args, &mut env).unwrap();
        if let Value::Symbol(s) = &result {
            assert_eq!(s, "abc");
        } else {
            panic!("expected Symbol, got {:?}", result);
        }
    }

    #[test]
    fn dispatch_cat_mixed() {
        let mut env = make_env();
        let args = vec![
            Value::Symbol("x".to_string()),
            Value::Integer(QInt::from(42i64)),
        ];
        let result = dispatch("cat", &args, &mut env).unwrap();
        if let Value::Symbol(s) = &result {
            assert_eq!(s, "x42");
        } else {
            panic!("expected Symbol, got {:?}", result);
        }
    }

    // -----------------------------------------------------------------------
    // Integration tests (parse + eval) -- Series Coefficient & Utility
    // -----------------------------------------------------------------------

    #[test]
    fn eval_coeff_constant() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("coeff(42, q, 0)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "42");
    }

    #[test]
    fn eval_degree_constant() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("degree(42, q)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "0");
    }

    #[test]
    fn eval_modp_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("modp(7, 3)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "1");
    }

    #[test]
    fn eval_mods_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("mods(5, 3)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "-1");
    }

    #[test]
    fn eval_type_integer() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("type(42, integer)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "true");
    }

    #[test]
    fn eval_evalb_comparison() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("evalb(3 > 2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "true");
    }

    #[test]
    fn eval_cat_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("cat(a, b, c)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "abc");
    }

    #[test]
    fn eval_numer_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("numer(3/4)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "3");
    }

    #[test]
    fn eval_denom_expr() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("denom(3/4)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "4");
    }

    // --- Iteration: add/mul/seq tests ---

    #[test]
    fn eval_add_sum_of_squares() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("add(i^2, i=1..5)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "55");
    }

    #[test]
    fn eval_add_empty_range() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("add(i, i=1..0)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "0");
    }

    #[test]
    fn eval_mul_factorial() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("mul(i, i=1..5)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "120");
    }

    #[test]
    fn eval_mul_empty_range() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("mul(i, i=1..0)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "1");
    }

    #[test]
    fn eval_seq_list_of_squares() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("seq(i^2, i=1..5)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "[1, 4, 9, 16, 25]");
    }

    #[test]
    fn eval_seq_empty_range() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("seq(i, i=1..0)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "[]");
    }

    #[test]
    fn eval_add_variable_scoping() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        // Set i:=99, then add(i, i=1..3), then check i is still 99
        let stmts = parse("i:=99: add(i, i=1..3): i").unwrap();
        // Execute all statements, collect last result
        for stmt in &stmts {
            eval_stmt(stmt, &mut env).unwrap();
        }
        // The last result should be i = 99
        let stmts2 = parse("i").unwrap();
        let result = eval_stmt(&stmts2[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "99");
    }

    #[test]
    fn eval_add_negative_bounds() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        // add(i, i=-2..2) = -2 + -1 + 0 + 1 + 2 = 0
        let stmts = parse("add(i, i=-2..2)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result, &env.symbols);
        assert_eq!(text, "0");
    }

    #[test]
    fn eval_range_outside_iteration_error() {
        use crate::parser::parse;
        let mut env = make_env();
        let stmts = parse("1..5").unwrap();
        let result = eval_stmt(&stmts[0], &mut env);
        assert!(result.is_err(), "1..5 at top level should error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("range expressions"),
            "error should mention range expressions, got: {}",
            err_msg
        );
    }
}
