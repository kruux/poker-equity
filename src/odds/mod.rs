mod chunk;
mod request;
mod runner;

pub use chunk::{ChunkResult, PlayerEquity};
pub use request::{run_chunk, run_exact, run_exact_within, EquityRequest, EXACT_DEAL_LIMIT};
pub use runner::{
    default_threads, equity, equity_with_progress, run_batch, Progress, Target,
};

#[cfg(test)]
mod tests {
    mod calibration;
    mod support;
    mod chunk_tests;
    mod weighted_tests;
    mod request_tests;
    mod runner_tests;
    mod deuce_equity_tests;
    mod holdem_equity_tests;
    mod omaha_equity_tests;
    mod razz_equity_tests;
    mod stud_equity_tests;
    mod stud_hi_lo_equity_tests;
}
