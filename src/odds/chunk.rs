/// What one run of the sampler measured.
///
/// Everything here is a **sum**, never an average, so two chunks merge by
/// addition and a caller can stop whenever it likes. The sum of squares is
/// carried for the same reason: without it a user reads a 0.3% gap between
/// two hands as meaningful, and a confidence interval that costs nothing to
/// keep is worth keeping.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkResult {
    /// How many deals went into these sums.
    pub samples: u64,
    /// Each seat's share of the pot, summed over the deals.
    pub share_sum: Vec<f64>,
    /// The same shares squared, for the standard error.
    pub share_square_sum: Vec<f64>,
    /// Deals where the seat took the whole pot.
    pub win_count: Vec<u64>,
    /// Deals where the seat took part of the pot but not all of it.
    pub tie_count: Vec<u64>,
    /// Deals where the seat took the whole pot, which in a split game means
    /// both halves.
    pub scoop_count: Vec<u64>,
    /// The low half alone, summed. Zero in games without one.
    pub low_share_sum: Vec<f64>,
    /// Whether these numbers come from enumerating every deal rather than
    /// sampling. An exact result has no error bar.
    pub exact: bool,
}

impl ChunkResult {
    /// An empty result for `seats` players.
    pub fn empty(seats: usize) -> Self {
        Self {
            samples: 0,
            share_sum: vec![0.0; seats],
            share_square_sum: vec![0.0; seats],
            win_count: vec![0; seats],
            tie_count: vec![0; seats],
            scoop_count: vec![0; seats],
            low_share_sum: vec![0.0; seats],
            exact: false,
        }
    }

    /// How many seats this covers.
    pub fn seats(&self) -> usize {
        self.share_sum.len()
    }

    /// Folds `other` into this one.
    ///
    /// The merged result is exact only when both parts were, since mixing a
    /// sample into an enumeration makes the whole thing an estimate.
    pub fn merge(&mut self, other: &ChunkResult) {
        debug_assert_eq!(self.seats(), other.seats(), "chunks must cover the same seats");
        self.samples += other.samples;
        self.exact = self.exact && other.exact;
        for seat in 0..self.seats() {
            self.share_sum[seat] += other.share_sum[seat];
            self.share_square_sum[seat] += other.share_square_sum[seat];
            self.win_count[seat] += other.win_count[seat];
            self.tie_count[seat] += other.tie_count[seat];
            self.scoop_count[seat] += other.scoop_count[seat];
            self.low_share_sum[seat] += other.low_share_sum[seat];
        }
    }

    /// Records one deal's outcome.
    pub(crate) fn record(&mut self, shares: &[f64], low_shares: &[f64]) {
        self.samples += 1;
        for seat in 0..self.seats() {
            let share = shares[seat];
            self.share_sum[seat] += share;
            self.share_square_sum[seat] += share * share;
            self.low_share_sum[seat] += low_shares[seat];
            if share >= 1.0 - f64::EPSILON {
                self.win_count[seat] += 1;
                self.scoop_count[seat] += 1;
            } else if share > 0.0 {
                self.tie_count[seat] += 1;
            }
        }
    }

    /// The finished numbers, one per seat.
    pub fn equities(&self) -> Vec<PlayerEquity> {
        (0..self.seats())
            .map(|seat| {
                let samples = self.samples.max(1) as f64;
                let equity = self.share_sum[seat] / samples;
                // The standard error of a mean: the spread of the per-deal
                // shares divided by the root of how many there were. Exactly
                // zero when the deals were enumerated rather than sampled.
                let std_error = if self.exact || self.samples < 2 {
                    0.0
                } else {
                    let mean_square = self.share_square_sum[seat] / samples;
                    let variance = (mean_square - equity * equity).max(0.0);
                    (variance / samples).sqrt()
                };
                PlayerEquity {
                    equity,
                    win: self.win_count[seat] as f64 / samples,
                    tie: self.tie_count[seat] as f64 / samples,
                    low_equity: self.low_share_sum[seat] / samples,
                    scoop: self.scoop_count[seat] as f64 / samples,
                    std_error,
                }
            })
            .collect()
    }
}

/// One seat's result.
///
/// `equity` leads because it is the answer; wins and ties are colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerEquity {
    /// Share of the pot, in `0..=1`. The number that matters.
    pub equity: f64,
    /// How often the seat took the whole pot.
    pub win: f64,
    /// How often it took part of one.
    pub tie: f64,
    /// The low half alone.
    pub low_equity: f64,
    /// How often it took both halves.
    pub scoop: f64,
    /// One standard error on `equity`; zero when the result is exact.
    pub std_error: f64,
}

impl PlayerEquity {
    /// `equity` as a percentage, which is how a table shows it.
    pub fn percent(&self) -> f64 {
        self.equity * 100.0
    }

    /// The half-width of a 95% confidence interval, as a percentage.
    pub fn margin_percent(&self) -> f64 {
        self.std_error * 1.96 * 100.0
    }
}
