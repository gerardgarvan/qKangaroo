//! Rendering subsystem for expressions (LaTeX and Unicode).
//!
//! Two backends:
//! - **LaTeX** (`to_latex`): Produces LaTeX strings following DLMF 17.2 notation
//!   for q-Pochhammer and basic hypergeometric series.
//! - **Unicode** (`DisplayExpr`): Implements `fmt::Display` for terminal rendering
//!   with Greek characters and subscript/superscript digits.

pub mod latex;
pub mod unicode;

pub use latex::to_latex;
pub use unicode::DisplayExpr;
