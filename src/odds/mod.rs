mod calculator;

pub use calculator::{EquityCalculator, SimulationProgress};

#[cfg(test)]
mod tests {
    mod deuce_calculator_tests;
    mod holdem_calculator_tests;
    mod omaha_calculator_tests;
    mod razz_calculator_tests;
    mod stud_calculator_tests;
    mod stud_hi_lo_calculator_tests;
}
