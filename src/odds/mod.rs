mod calculator;
mod chunk;
mod request;
mod runner;

pub use calculator::{EquityCalculator, SimulationProgress};
pub use chunk::{ChunkResult, PlayerEquity};
pub use request::{run_chunk, run_exact, run_exact_within, EquityRequest, EXACT_DEAL_LIMIT};
pub use runner::{equity, equity_with_progress, Progress, Target};

#[cfg(test)]
mod tests {
    mod chunk_tests;
    mod runner_tests;
    mod deuce_calculator_tests;
    mod holdem_calculator_tests;
    mod omaha_calculator_tests;
    mod razz_calculator_tests;
    mod stud_calculator_tests;
    mod stud_hi_lo_calculator_tests;
}
