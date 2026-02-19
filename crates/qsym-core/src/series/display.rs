//! Display implementation for FormalPowerSeries.
//!
//! Format: "1 - q + 2*q^3 + O(q^10)"
//! Zero series: "O(q^10)"
//! Uses "q" as variable name (hardcoded for now).

use std::cmp::Ordering;
use std::fmt;

use super::FormalPowerSeries;

impl fmt::Display for FormalPowerSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let var = "q";
        let mut first = true;

        for (&k, c) in &self.coefficients {
            let is_negative = c.0.cmp0() == Ordering::Less;
            let abs_c = if is_negative {
                -c.clone()
            } else {
                c.clone()
            };

            let abs_is_one = abs_c.0.numer().cmp0() != Ordering::Equal
                && *abs_c.0.numer() == *abs_c.0.denom();

            if first {
                // First term: show sign directly if negative
                if is_negative {
                    write!(f, "-")?;
                }
                if k == 0 {
                    // Constant term: just show the value
                    write!(f, "{}", abs_c)?;
                } else if abs_is_one {
                    // Coefficient is 1 or -1: show just the variable part
                    if k == 1 {
                        write!(f, "{}", var)?;
                    } else {
                        write!(f, "{}^{}", var, k)?;
                    }
                } else {
                    // General coefficient
                    if k == 1 {
                        write!(f, "{}*{}", abs_c, var)?;
                    } else {
                        write!(f, "{}*{}^{}", abs_c, var, k)?;
                    }
                }
                first = false;
            } else {
                // Subsequent terms: use " + " or " - "
                if is_negative {
                    write!(f, " - ")?;
                } else {
                    write!(f, " + ")?;
                }
                if k == 0 {
                    write!(f, "{}", abs_c)?;
                } else if abs_is_one {
                    if k == 1 {
                        write!(f, "{}", var)?;
                    } else {
                        write!(f, "{}^{}", var, k)?;
                    }
                } else {
                    if k == 1 {
                        write!(f, "{}*{}", abs_c, var)?;
                    } else {
                        write!(f, "{}*{}^{}", abs_c, var, k)?;
                    }
                }
            }
        }

        // Append truncation order (suppress for polynomial sentinel)
        if self.truncation_order < 1_000_000_000 {
            if first {
                // No terms were written (zero series)
                write!(f, "O({}^{})", var, self.truncation_order)?;
            } else {
                write!(f, " + O({}^{})", var, self.truncation_order)?;
            }
        } else if first {
            // Polynomial with zero terms
            write!(f, "0")?;
        }

        Ok(())
    }
}
