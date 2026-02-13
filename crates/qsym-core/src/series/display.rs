//! Display implementation for FormalPowerSeries.
//!
//! Format: "1 - q + 2*q^3 + O(q^10)"
//! Zero series: "O(q^10)"
//! Uses "q" as variable name (hardcoded for now).

use std::fmt;
use super::FormalPowerSeries;

impl fmt::Display for FormalPowerSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Display for FormalPowerSeries not yet implemented")
    }
}
