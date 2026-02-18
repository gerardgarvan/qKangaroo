//! Evaluator core for the q-Kangaroo REPL.
//!
//! Walks [`AstNode`] trees, manages a variable [`Environment`], performs
//! arithmetic on [`Value`] types, catches panics from qsym-core, and
//! dispatches function calls.

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

use qsym_core::number::{QInt, QRat};
use qsym_core::series::arithmetic;
use qsym_core::series::FormalPowerSeries;

use crate::ast::{AstNode, BinOp, Stmt, Terminator};
use crate::environment::Environment;

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
    /// None/null (try_summation returns None on failure).
    None,
    /// The infinity keyword.
    Infinity,
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
            Value::None => "none",
            Value::Infinity => "infinity",
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
            signature: String::new(),
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
            signature: String::new(),
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

/// Evaluate a statement with panic catching.
///
/// Wraps [`eval_stmt`] in `catch_unwind` with `AssertUnwindSafe`.
/// On panic, extracts the message and returns `EvalError::Panic`.
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
            Err(EvalError::Panic(msg))
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

        AstNode::Q => {
            // Create the monomial series q^1 + O(q^N)
            let fps = FormalPowerSeries::monomial(
                env.sym_q,
                QRat::one(),
                1,
                env.default_order,
            );
            Ok(Value::Series(fps))
        }

        AstNode::Infinity => Ok(Value::Infinity),

        AstNode::LastResult => match &env.last_result {
            Some(val) => Ok(val.clone()),
            None => Err(EvalError::NoLastResult),
        },

        AstNode::Variable(name) => match env.get_var(name) {
            Some(val) => Ok(val.clone()),
            None => Err(EvalError::UnknownVariable {
                name: name.clone(),
            }),
        },

        AstNode::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                result.push(eval_expr(item, env)?);
            }
            Ok(Value::List(result))
        }

        AstNode::Neg(inner) => {
            let val = eval_expr(inner, env)?;
            eval_negate(val)
        }

        AstNode::BinOp { op, lhs, rhs } => {
            let left = eval_expr(lhs, env)?;
            let right = eval_expr(rhs, env)?;
            eval_binop(*op, left, right, env)
        }

        AstNode::FuncCall { name, args } => {
            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(eval_expr(arg, env)?);
            }
            dispatch(name, &evaluated, env)
        }

        AstNode::Assign { name, value } => {
            let val = eval_expr(value, env)?;
            env.set_var(name, val.clone());
            Ok(val)
        }
    }
}

// ---------------------------------------------------------------------------
// Unary negation
// ---------------------------------------------------------------------------

/// Negate a value.
fn eval_negate(val: Value) -> Result<Value, EvalError> {
    match val {
        Value::Series(fps) => Ok(Value::Series(arithmetic::negate(&fps))),
        Value::Integer(n) => Ok(Value::Integer(-n)),
        Value::Rational(r) => Ok(Value::Rational(-r)),
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
    env: &Environment,
) -> Result<Value, EvalError> {
    match op {
        BinOp::Add => eval_add(left, right, env),
        BinOp::Sub => eval_sub(left, right, env),
        BinOp::Mul => eval_mul(left, right, env),
        BinOp::Div => eval_div(left, right, env),
        BinOp::Pow => eval_pow(left, right),
    }
}

/// Convert a numeric value (Integer or Rational) to a constant FPS.
fn value_to_constant_fps(val: &Value, env: &Environment) -> Option<FormalPowerSeries> {
    let qrat = match val {
        Value::Integer(n) => QRat::from(n.clone()),
        Value::Rational(r) => r.clone(),
        _ => return None,
    };
    Some(FormalPowerSeries::monomial(env.sym_q, qrat, 0, env.default_order))
}

/// Convert a numeric value to QRat for scalar operations.
fn value_to_qrat(val: &Value) -> Option<QRat> {
    match val {
        Value::Integer(n) => Some(QRat::from(n.clone())),
        Value::Rational(r) => Some(r.clone()),
        _ => None,
    }
}

fn eval_add(left: Value, right: Value, env: &Environment) -> Result<Value, EvalError> {
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
        // Series + scalar: promote scalar to constant FPS
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let const_fps = value_to_constant_fps(&right, env).unwrap();
            Ok(Value::Series(arithmetic::add(fps, &const_fps)))
        }
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let const_fps = value_to_constant_fps(&left, env).unwrap();
            Ok(Value::Series(arithmetic::add(&const_fps, fps)))
        }
        _ => Err(EvalError::TypeError {
            operation: "+".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_sub(left: Value, right: Value, env: &Environment) -> Result<Value, EvalError> {
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
        // Series - scalar
        (Value::Series(fps), _) if value_to_qrat(&right).is_some() => {
            let const_fps = value_to_constant_fps(&right, env).unwrap();
            Ok(Value::Series(arithmetic::sub(fps, &const_fps)))
        }
        (_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
            let const_fps = value_to_constant_fps(&left, env).unwrap();
            Ok(Value::Series(arithmetic::sub(&const_fps, fps)))
        }
        _ => Err(EvalError::TypeError {
            operation: "-".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_mul(left: Value, right: Value, _env: &Environment) -> Result<Value, EvalError> {
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
        _ => Err(EvalError::TypeError {
            operation: "*".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_div(left: Value, right: Value, env: &Environment) -> Result<Value, EvalError> {
    match (&left, &right) {
        (Value::Series(a), Value::Series(b)) => {
            let inv = arithmetic::invert(b);
            Ok(Value::Series(arithmetic::mul(a, &inv)))
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
            let const_fps = value_to_constant_fps(&left, env).unwrap();
            let inv = arithmetic::invert(fps);
            Ok(Value::Series(arithmetic::mul(&const_fps, &inv)))
        }
        _ => Err(EvalError::TypeError {
            operation: "/".to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
        }),
    }
}

fn eval_pow(left: Value, right: Value) -> Result<Value, EvalError> {
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

// ---------------------------------------------------------------------------
// Function dispatch (stub for Plans 02/03)
// ---------------------------------------------------------------------------

/// Dispatch a function call by name.
///
/// This is a stub that handles alias resolution and fuzzy matching.
/// Plans 02 and 03 will replace the body with the full dispatch table
/// for all 79 functions.
pub fn dispatch(
    name: &str,
    _args: &[Value],
    _env: &mut Environment,
) -> Result<Value, EvalError> {
    let canonical = resolve_alias(name);

    // For now, all functions are unknown (Plans 02/03 fill in dispatch)
    let suggestions = find_similar_names(&canonical);
    Err(EvalError::UnknownFunction {
        name: name.to_string(),
        suggestions,
    })
}

// ---------------------------------------------------------------------------
// Alias resolution
// ---------------------------------------------------------------------------

/// Resolve a Maple alias to its canonical q-Kangaroo name.
///
/// Case-insensitive lookup. If no alias matches, returns the input unchanged.
fn resolve_alias(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "numbpart" => "partition_count".to_string(),
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
        _ => name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Fuzzy matching for "Did you mean?" suggestions
// ---------------------------------------------------------------------------

/// All canonical function names (79 functions) for fuzzy matching.
const ALL_FUNCTION_NAMES: &[&str] = &[
    // Pattern A: Series generators
    "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
    "theta2", "theta3", "theta4",
    "partition_gf", "distinct_parts_gf", "odd_parts_gf", "bounded_parts_gf",
    "rank_gf", "crank_gf",
    // Pattern B: No-session
    "partition_count",
    // Pattern C: Series-input analysis
    "sift", "qdegree", "lqdegree", "qfactor",
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
    // Pattern H: Identity proving
    "prove_eta_id", "search_identities",
    // Pattern I: Mock theta / Appell-Lerch
    "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3",
    "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
    "mock_theta_f5_0", "mock_theta_f5_1",
    "mock_theta_phi5_0", "mock_theta_phi5_1",
    "mock_theta_psi5_0", "mock_theta_psi5_1",
    "mock_theta_chi5_0", "mock_theta_chi5_1",
    "mock_theta_F7_0", "mock_theta_F7_1", "mock_theta_F7_2",
    "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
    // Pattern J: Bailey
    "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain", "bailey_discover",
    // Pattern K: Algorithmic
    "q_gosper", "q_zeilberger", "verify_wz", "q_petkovsek",
    // Pattern L: Nonterminating
    "prove_nonterminating", "find_transformation_chain",
];

/// All alias names for fuzzy matching.
const ALL_ALIAS_NAMES: &[&str] = &[
    "numbpart", "rankgf", "crankgf", "qphihyper", "qpsihyper",
    "qgauss", "proveid", "qzeil", "qzeilberger", "qpetkovsek",
    "qgosper", "findlincombo_modp", "findhom_modp", "findhomcombo_modp", "search_id",
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
    use crate::ast::{AstNode, BinOp, Stmt, Terminator};
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
    fn eval_q_creates_series() {
        let mut env = make_env();
        let val = eval_expr(&AstNode::Q, &mut env).unwrap();
        if let Value::Series(fps) = val {
            assert_eq!(fps.coeff(1), QRat::one());
            assert_eq!(fps.coeff(0), QRat::zero());
            assert_eq!(fps.truncation_order(), 20);
        } else {
            panic!("expected Series, got {:?}", val);
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
    fn eval_variable_not_found() {
        let mut env = make_env();
        let err = eval_expr(&AstNode::Variable("unknown".to_string()), &mut env).unwrap_err();
        assert!(matches!(err, EvalError::UnknownVariable { .. }));
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
        // q + q = 2*q
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Q),
            rhs: Box::new(AstNode::Q),
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
        // 3 * q -> 3*q
        let node = AstNode::BinOp {
            op: BinOp::Mul,
            lhs: Box::new(AstNode::Integer(3)),
            rhs: Box::new(AstNode::Q),
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
        // q + 1 -> 1 + q + O(q^20)
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Q),
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
        assert_eq!(resolve_alias("numbpart"), "partition_count");
    }

    #[test]
    fn resolve_alias_case_insensitive() {
        assert_eq!(resolve_alias("NUMBPART"), "partition_count");
        assert_eq!(resolve_alias("QZeil"), "q_zeilberger");
    }

    #[test]
    fn resolve_alias_passthrough() {
        assert_eq!(resolve_alias("aqprod"), "aqprod");
        assert_eq!(resolve_alias("etaq"), "etaq");
    }

    #[test]
    fn resolve_alias_all_maple_names() {
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
                    msg.contains("zero constant term"),
                    "expected panic about zero constant term, got: {}",
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
}
