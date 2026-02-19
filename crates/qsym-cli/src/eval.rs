//! Evaluator core for the q-Kangaroo REPL.
//!
//! Walks [`AstNode`] trees, manages a variable [`Environment`], performs
//! arithmetic on [`Value`] types, catches panics from qsym-core, and
//! dispatches function calls.

use std::collections::BTreeMap;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

use qsym_core::number::{QInt, QRat};
use qsym_core::qseries::{self, QMonomial, PochhammerOrder};
use qsym_core::qseries::{HypergeometricSeries, BilateralHypergeometricSeries};
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
    /// String value (for filenames, etc.).
    String(String),
    /// None/null (try_summation returns None on failure).
    None,
    /// The infinity keyword.
    Infinity,
    /// A symbolic variable name (undefined name fallback, Maple-like).
    Symbol(String),
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
            // aqprod(coeff_num, coeff_den, power, n_or_infinity, order)
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

        "qbin" => {
            // qbin(n, k, order)
            expect_args(name, args, 3)?;
            let n = extract_i64(name, args, 0)?;
            let k = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::qbin(n, k, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "etaq" => {
            // etaq(b, t, order)
            expect_args(name, args, 3)?;
            let b = extract_i64(name, args, 0)?;
            let t = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::etaq(b, t, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "jacprod" => {
            // jacprod(a, b, order)
            expect_args(name, args, 3)?;
            let a = extract_i64(name, args, 0)?;
            let b = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::jacprod(a, b, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "tripleprod" => {
            // tripleprod(coeff_num, coeff_den, power, order)
            expect_args(name, args, 4)?;
            let cn = extract_i64(name, args, 0)?;
            let cd = extract_i64(name, args, 1)?;
            let power = extract_i64(name, args, 2)?;
            let order = extract_i64(name, args, 3)?;
            let monomial = QMonomial::new(QRat::from((cn, cd)), power);
            let result = qseries::tripleprod(&monomial, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "quinprod" => {
            // quinprod(coeff_num, coeff_den, power, order)
            expect_args(name, args, 4)?;
            let cn = extract_i64(name, args, 0)?;
            let cd = extract_i64(name, args, 1)?;
            let power = extract_i64(name, args, 2)?;
            let order = extract_i64(name, args, 3)?;
            let monomial = QMonomial::new(QRat::from((cn, cd)), power);
            let result = qseries::quinprod(&monomial, env.sym_q, order);
            Ok(Value::Series(result))
        }

        "winquist" => {
            // winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)
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

        // =================================================================
        // Group 2: Partitions (FUNC-02) -- 7 functions
        // =================================================================

        "partition_count" => {
            // partition_count(n)
            expect_args(name, args, 1)?;
            let n = extract_i64(name, args, 0)?;
            let result = qseries::partition_count(n);
            // partition_count returns QRat (always integer-valued)
            Ok(Value::Integer(QInt(result.0.numer().clone())))
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
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::theta2(env.sym_q, order);
            Ok(Value::Series(result))
        }

        "theta3" => {
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::theta3(env.sym_q, order);
            Ok(Value::Series(result))
        }

        "theta4" => {
            expect_args(name, args, 1)?;
            let order = extract_i64(name, args, 0)?;
            let result = qseries::theta4(env.sym_q, order);
            Ok(Value::Series(result))
        }

        // =================================================================
        // Group 4: Series Analysis (FUNC-04) -- 9 functions
        // =================================================================

        "sift" => {
            // sift(series, m, j)
            expect_args(name, args, 3)?;
            let fps = extract_series(name, args, 0)?;
            let m = extract_i64(name, args, 1)?;
            let j = extract_i64(name, args, 2)?;
            let result = qseries::sift(&fps, m, j);
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

        "prodmake" => {
            // prodmake(series, max_n)
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let max_n = extract_i64(name, args, 1)?;
            let result = qseries::prodmake(&fps, max_n);
            Ok(infinite_product_form_to_value(&result))
        }

        "etamake" => {
            // etamake(series, max_n)
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let max_n = extract_i64(name, args, 1)?;
            let result = qseries::etamake(&fps, max_n);
            Ok(eta_quotient_to_value(&result))
        }

        "jacprodmake" => {
            // jacprodmake(series, max_n)
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let max_n = extract_i64(name, args, 1)?;
            let result = qseries::jacprodmake(&fps, max_n);
            Ok(jacobi_product_form_to_value(&result))
        }

        "mprodmake" => {
            // mprodmake(series, max_n)
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let max_n = extract_i64(name, args, 1)?;
            let result = qseries::mprodmake(&fps, max_n);
            Ok(btreemap_i64_to_value(&result))
        }

        "qetamake" => {
            // qetamake(series, max_n)
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let max_n = extract_i64(name, args, 1)?;
            let result = qseries::qetamake(&fps, max_n);
            Ok(q_eta_form_to_value(&result))
        }

        "qfactor" => {
            // qfactor(series)
            expect_args(name, args, 1)?;
            let fps = extract_series(name, args, 0)?;
            let result = qseries::qfactor(&fps);
            Ok(q_factorization_to_value(&result))
        }

        // =================================================================
        // Group 5: Relation Discovery (FUNC-05) -- 15 functions
        // =================================================================

        // Pattern D: target + list of candidates

        "findlincombo" => {
            // findlincombo(target, [candidates...], topshift)
            expect_args(name, args, 3)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let topshift = extract_i64(name, args, 2)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findlincombo(&target, &refs, topshift) {
                Some(coeffs) => Ok(Value::List(coeffs.into_iter().map(Value::Rational).collect())),
                None => Ok(Value::None),
            }
        }

        "findhomcombo" => {
            // findhomcombo(target, [candidates...], degree, topshift)
            expect_args(name, args, 4)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let degree = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findhomcombo(&target, &refs, degree, topshift) {
                Some(coeffs) => Ok(Value::List(coeffs.into_iter().map(Value::Rational).collect())),
                None => Ok(Value::None),
            }
        }

        "findnonhomcombo" => {
            // findnonhomcombo(target, [candidates...], degree, topshift)
            expect_args(name, args, 4)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let degree = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findnonhomcombo(&target, &refs, degree, topshift) {
                Some(coeffs) => Ok(Value::List(coeffs.into_iter().map(Value::Rational).collect())),
                None => Ok(Value::None),
            }
        }

        "findlincombomodp" => {
            // findlincombomodp(target, [candidates...], p, topshift)
            expect_args(name, args, 4)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let p = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findlincombomodp(&target, &refs, p, topshift) {
                Some(coeffs) => Ok(Value::List(
                    coeffs.into_iter().map(|c| Value::Integer(QInt::from(c))).collect(),
                )),
                None => Ok(Value::None),
            }
        }

        "findhomcombomodp" => {
            // findhomcombomodp(target, [candidates...], p, degree, topshift)
            expect_args(name, args, 5)?;
            let target = extract_series(name, args, 0)?;
            let candidates = extract_series_list(name, args, 1)?;
            let p = extract_i64(name, args, 2)?;
            let degree = extract_i64(name, args, 3)?;
            let topshift = extract_i64(name, args, 4)?;
            let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
            match qseries::findhomcombomodp(&target, &refs, p, degree, topshift) {
                Some(coeffs) => Ok(Value::List(
                    coeffs.into_iter().map(|c| Value::Integer(QInt::from(c))).collect(),
                )),
                None => Ok(Value::None),
            }
        }

        // Pattern E: list of series

        "findhom" => {
            // findhom([series...], degree, topshift)
            expect_args(name, args, 3)?;
            let series_list = extract_series_list(name, args, 0)?;
            let degree = extract_i64(name, args, 1)?;
            let topshift = extract_i64(name, args, 2)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findhom(&refs, degree, topshift);
            Ok(Value::List(
                rows.into_iter()
                    .map(|row| Value::List(row.into_iter().map(Value::Rational).collect()))
                    .collect(),
            ))
        }

        "findnonhom" => {
            // findnonhom([series...], degree, topshift)
            expect_args(name, args, 3)?;
            let series_list = extract_series_list(name, args, 0)?;
            let degree = extract_i64(name, args, 1)?;
            let topshift = extract_i64(name, args, 2)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findnonhom(&refs, degree, topshift);
            Ok(Value::List(
                rows.into_iter()
                    .map(|row| Value::List(row.into_iter().map(Value::Rational).collect()))
                    .collect(),
            ))
        }

        "findhommodp" => {
            // findhommodp([series...], p, degree, topshift)
            expect_args(name, args, 4)?;
            let series_list = extract_series_list(name, args, 0)?;
            let p = extract_i64(name, args, 1)?;
            let degree = extract_i64(name, args, 2)?;
            let topshift = extract_i64(name, args, 3)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let rows = qseries::findhommodp(&refs, p, degree, topshift);
            Ok(Value::List(
                rows.into_iter()
                    .map(|row| Value::List(
                        row.into_iter().map(|c| Value::Integer(QInt::from(c))).collect(),
                    ))
                    .collect(),
            ))
        }

        "findmaxind" => {
            // findmaxind([series...], topshift)
            expect_args(name, args, 2)?;
            let series_list = extract_series_list(name, args, 0)?;
            let topshift = extract_i64(name, args, 1)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let indices = qseries::findmaxind(&refs, topshift);
            Ok(Value::List(
                indices.into_iter().map(|i| Value::Integer(QInt::from(i as i64))).collect(),
            ))
        }

        "findprod" => {
            // findprod([series...], max_coeff, max_exp)
            expect_args(name, args, 3)?;
            let series_list = extract_series_list(name, args, 0)?;
            let max_coeff = extract_i64(name, args, 1)?;
            let max_exp = extract_i64(name, args, 2)?;
            let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
            let results = qseries::findprod(&refs, max_coeff, max_exp);
            Ok(Value::List(
                results.into_iter()
                    .map(|row| Value::List(
                        row.into_iter().map(|c| Value::Integer(QInt::from(c))).collect(),
                    ))
                    .collect(),
            ))
        }

        "findcong" => {
            // findcong(series, [moduli...])
            expect_args(name, args, 2)?;
            let fps = extract_series(name, args, 0)?;
            let moduli = extract_i64_list(name, args, 1)?;
            let results = qseries::findcong(&fps, &moduli);
            Ok(Value::List(
                results.into_iter()
                    .map(|c| congruence_to_value(&c))
                    .collect(),
            ))
        }

        // Pattern F: two series

        "findpoly" => {
            // findpoly(x, y, deg_x, deg_y, topshift)
            expect_args(name, args, 5)?;
            let x = extract_series(name, args, 0)?;
            let y = extract_series(name, args, 1)?;
            let deg_x = extract_i64(name, args, 2)?;
            let deg_y = extract_i64(name, args, 3)?;
            let topshift = extract_i64(name, args, 4)?;
            match qseries::findpoly(&x, &y, deg_x, deg_y, topshift) {
                Some(rel) => Ok(polynomial_relation_to_value(&rel)),
                None => Ok(Value::None),
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
    let mut factor_entries: Vec<(String, Value)> = Vec::new();
    for (&d, &r_d) in &eq.factors {
        factor_entries.push((d.to_string(), Value::Integer(QInt::from(r_d))));
    }
    Value::Dict(vec![
        ("factors".to_string(), Value::Dict(factor_entries)),
        ("q_shift".to_string(), Value::Rational(eq.q_shift.clone())),
    ])
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

/// Convert a `QFactorization` to `Value::Dict`.
fn q_factorization_to_value(qf: &qseries::QFactorization) -> Value {
    let mut factor_entries: Vec<(String, Value)> = Vec::new();
    for (&i, &mult) in &qf.factors {
        factor_entries.push((i.to_string(), Value::Integer(QInt::from(mult))));
    }
    Value::Dict(vec![
        ("scalar".to_string(), Value::Rational(qf.scalar.clone())),
        ("factors".to_string(), Value::Dict(factor_entries)),
        ("is_exact".to_string(), Value::Bool(qf.is_exact)),
    ])
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
        "aqprod" => "(coeff_num, coeff_den, power, n_or_infinity, order)".to_string(),
        "qbin" => "(n, k, order)".to_string(),
        "etaq" => "(b, t, order)".to_string(),
        "jacprod" => "(a, b, order)".to_string(),
        "tripleprod" => "(coeff_num, coeff_den, power, order)".to_string(),
        "quinprod" => "(coeff_num, coeff_den, power, order)".to_string(),
        "winquist" => "(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)".to_string(),
        // Group 2: Partitions
        "partition_count" => "(n)".to_string(),
        "partition_gf" => "(order)".to_string(),
        "distinct_parts_gf" => "(order)".to_string(),
        "odd_parts_gf" => "(order)".to_string(),
        "bounded_parts_gf" => "(max_part, order)".to_string(),
        "rank_gf" => "(z_num, z_den, order)".to_string(),
        "crank_gf" => "(z_num, z_den, order)".to_string(),
        // Group 3: Theta Functions
        "theta2" => "(order)".to_string(),
        "theta3" => "(order)".to_string(),
        "theta4" => "(order)".to_string(),
        // Group 4: Series Analysis
        "sift" => "(series, m, j)".to_string(),
        "qdegree" => "(series)".to_string(),
        "lqdegree" => "(series)".to_string(),
        "prodmake" => "(series, max_n)".to_string(),
        "etamake" => "(series, max_n)".to_string(),
        "jacprodmake" => "(series, max_n)".to_string(),
        "mprodmake" => "(series, max_n)".to_string(),
        "qetamake" => "(series, max_n)".to_string(),
        "qfactor" => "(series)".to_string(),
        // Group 5: Relation Discovery
        "findlincombo" => "(target, [candidates], topshift)".to_string(),
        "findhomcombo" => "(target, [candidates], degree, topshift)".to_string(),
        "findnonhomcombo" => "(target, [candidates], degree, topshift)".to_string(),
        "findlincombomodp" => "(target, [candidates], p, topshift)".to_string(),
        "findhomcombomodp" => "(target, [candidates], p, degree, topshift)".to_string(),
        "findhom" => "([series], degree, topshift)".to_string(),
        "findnonhom" => "([series], degree, topshift)".to_string(),
        "findhommodp" => "([series], p, degree, topshift)".to_string(),
        "findmaxind" => "([series], topshift)".to_string(),
        "findprod" => "([series], max_coeff, max_exp)".to_string(),
        "findcong" => "(series, [moduli])".to_string(),
        "findpoly" => "(x, y, deg_x, deg_y, topshift)".to_string(),
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
        "g2" => "universal_mock_theta_g2".to_string(),
        "g3" => "universal_mock_theta_g3".to_string(),
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
];

/// All alias names for fuzzy matching.
const ALL_ALIAS_NAMES: &[&str] = &[
    "numbpart", "rankgf", "crankgf", "qphihyper", "qpsihyper",
    "qgauss", "proveid", "qzeil", "qzeilberger", "qpetkovsek",
    "qgosper", "findlincombo_modp", "findhom_modp", "findhomcombo_modp", "search_id",
    "g2", "g3",
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

    // --- Dispatch: Group 2 (Partitions) ---

    #[test]
    fn dispatch_partition_count_5() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(5i64))];
        let val = dispatch("partition_count", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(7i64));
        } else {
            panic!("expected Integer, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_partition_count_50() {
        let mut env = make_env();
        let args = vec![Value::Integer(QInt::from(50i64))];
        let val = dispatch("partition_count", &args, &mut env).unwrap();
        if let Value::Integer(n) = val {
            assert_eq!(n, QInt::from(204226i64));
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
        // Build a partition_gf first, then sift it
        let args = vec![Value::Integer(QInt::from(30i64))];
        let pgf = dispatch("partition_gf", &args, &mut env).unwrap();
        let sift_args = vec![
            pgf,
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(0i64)),
        ];
        let val = dispatch("sift", &sift_args, &mut env).unwrap();
        assert!(matches!(val, Value::Series(_)));
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
        // prodmake on partition_gf should give exponents a_n = 1 for all n
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("prodmake", &[pgf, Value::Integer(QInt::from(10i64))], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            // Should have "exponents" and "terms_used" keys
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"exponents"), "expected 'exponents' in {:?}", keys);
            assert!(keys.contains(&"terms_used"), "expected 'terms_used' in {:?}", keys);
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_etamake_returns_dict() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("etamake", &[pgf, Value::Integer(QInt::from(10i64))], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"factors"), "expected 'factors' in {:?}", keys);
            assert!(keys.contains(&"q_shift"), "expected 'q_shift' in {:?}", keys);
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_jacprodmake_returns_dict() {
        let mut env = make_env();
        let jp = dispatch("jacprod", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("jacprodmake", &[jp, Value::Integer(QInt::from(10i64))], &mut env).unwrap();
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
        // distinct_parts_gf = prod(1+q^n) -- mprodmake should decompose it
        let dp = dispatch("distinct_parts_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("mprodmake", &[dp, Value::Integer(QInt::from(10i64))], &mut env).unwrap();
        assert!(matches!(val, Value::Dict(_)));
    }

    #[test]
    fn dispatch_qetamake_returns_dict() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let val = dispatch("qetamake", &[pgf, Value::Integer(QInt::from(10i64))], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"factors"));
            assert!(keys.contains(&"q_shift"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_qfactor_returns_dict_with_is_exact() {
        let mut env = make_env();
        // qbin(5,2,20) is a polynomial: (1-q^4)(1-q^5)/((1-q)(1-q^2))
        let qb = dispatch("qbin", &[
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let val = dispatch("qfactor", &[qb], &mut env).unwrap();
        if let Value::Dict(entries) = &val {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"scalar"));
            assert!(keys.contains(&"factors"));
            assert!(keys.contains(&"is_exact"));
        } else {
            panic!("expected Dict, got {:?}", val);
        }
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
        let text = format_value(&val);
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
        let text = format_value(&result);
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

        // prodmake(f, 10)
        let stmts2 = parse("prodmake(f, 10)").unwrap();
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
        let text = format_value(&result);
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

        // g := sift(f, 5, 4)
        let stmts2 = parse("g := sift(f, 5, 4)").unwrap();
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

        // qfactor(qbin(5, 2, 20)) returns a Dict with scalar, factors, is_exact
        let stmts = parse("qfactor(qbin(5, 2, 20))").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        if let Value::Dict(entries) = &result {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"scalar"), "expected 'scalar' key");
            assert!(keys.contains(&"factors"), "expected 'factors' key");
            assert!(keys.contains(&"is_exact"), "expected 'is_exact' key");
        } else {
            panic!("expected Dict");
        }
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
    fn dispatch_findlincombo_returns_list_or_none() {
        let mut env = make_env();
        // Build two simple series: partition_gf and etaq(1,1,20)
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(20i64))], &mut env).unwrap();
        let etq = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let candidates = Value::List(vec![pgf.clone(), etq.clone()]);
        let args = vec![pgf, candidates, Value::Integer(QInt::from(5i64))];
        let val = dispatch("findlincombo", &args, &mut env).unwrap();
        // Should find a combination (first basis is identical to target)
        match val {
            Value::List(coeffs) => {
                assert_eq!(coeffs.len(), 2);
                // First coefficient should be 1 (target == first basis element)
            }
            Value::None => {} // also acceptable depending on truncation
            other => panic!("expected List or None, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_findhom_returns_matrix() {
        let mut env = make_env();
        let e1 = dispatch("etaq", &[
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(20i64)),
        ], &mut env).unwrap();
        let series_list = Value::List(vec![e1]);
        let args = vec![
            series_list,
            Value::Integer(QInt::from(2i64)),  // degree
            Value::Integer(QInt::from(5i64)),  // topshift
        ];
        let val = dispatch("findhom", &args, &mut env).unwrap();
        assert!(matches!(val, Value::List(_)));
    }

    #[test]
    fn dispatch_findcong_returns_list_of_dicts() {
        let mut env = make_env();
        let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(50i64))], &mut env).unwrap();
        let moduli = Value::List(vec![
            Value::Integer(QInt::from(5i64)),
            Value::Integer(QInt::from(7i64)),
        ]);
        let val = dispatch("findcong", &[pgf, moduli], &mut env).unwrap();
        if let Value::List(congruences) = val {
            // Should find at least the p(5n+4) congruence
            assert!(!congruences.is_empty(), "expected at least one congruence");
            // Each entry should be a dict
            for cong in &congruences {
                if let Value::Dict(entries) = cong {
                    let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
                    assert!(keys.contains(&"modulus"));
                    assert!(keys.contains(&"residue"));
                    assert!(keys.contains(&"divisor"));
                } else {
                    panic!("expected Dict in congruences list, got {:?}", cong);
                }
            }
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
        } else {
            panic!("expected List, got {:?}", val);
        }
    }

    #[test]
    fn dispatch_findpoly_returns_dict_or_none() {
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
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(5i64)),
        ];
        let val = dispatch("findpoly", &args, &mut env).unwrap();
        // Could be Dict (found relation) or None (no relation in that degree)
        match &val {
            Value::Dict(_) | Value::None => {}
            other => panic!("expected Dict or None, got {:?}", other),
        }
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
        let text = format_value(&result);
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
    fn integration_format_etaq_starts_with_1() {
        use crate::parser::parse;
        use crate::format::format_value;
        let mut env = make_env();
        let stmts = parse("etaq(1,1,20)").unwrap();
        let result = eval_stmt(&stmts[0], &mut env).unwrap().unwrap();
        let text = format_value(&result);
        assert!(text.starts_with("1"), "expected output starting with '1', got: {}", text);
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
        let text = format_value(&result);
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
            count >= 75,
            "expected at least 75 function names in ALL_FUNCTION_NAMES, got {}",
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
}
