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
    /// How many deals were attempted to get them.
    ///
    /// A deal is thrown away when the cards drawn cannot fill every slot --
    /// one seat's wildcard taking the last card another seat needed by name.
    /// Equal to `samples` when nothing was thrown away.
    pub attempts: u64,
    /// The deals' weights, summed. Every weight is one unless the sampler
    /// drew against the live deck, so this equals `samples` on the ordinary
    /// path and every average below is the plain average it always was.
    pub weight_sum: f64,
    /// The weights squared, summed. With [`weight_sum`](Self::weight_sum)
    /// this gives the effective sample size, which is what a weighted pile of
    /// deals is really worth.
    pub weight_square_sum: f64,
    /// Each seat's share of the pot, weighted and summed.
    pub share_sum: Vec<f64>,
    /// The shares squared, weighted by the *square* of the deal's weight,
    /// which is what the standard error needs.
    pub share_square_sum: Vec<f64>,
    /// The shares weighted by the square of the deal's weight. Carried only
    /// for the standard error; equal to `share_sum` when nothing is weighted.
    pub square_weight_share_sum: Vec<f64>,
    /// Deals where the seat took the whole pot, weighted.
    pub win_count: Vec<f64>,
    /// Deals where the seat took part of the pot but not all of it, weighted.
    pub tie_count: Vec<f64>,
    /// Deals where the seat took the whole pot, which in a split game means
    /// both halves. Weighted.
    pub scoop_count: Vec<f64>,
    /// The low half alone, weighted and summed. Zero in games without one.
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
            attempts: 0,
            weight_sum: 0.0,
            weight_square_sum: 0.0,
            share_sum: vec![0.0; seats],
            share_square_sum: vec![0.0; seats],
            square_weight_share_sum: vec![0.0; seats],
            win_count: vec![0.0; seats],
            tie_count: vec![0.0; seats],
            scoop_count: vec![0.0; seats],
            low_share_sum: vec![0.0; seats],
            exact: false,
        }
    }

    /// How many seats this covers.
    pub fn seats(&self) -> usize {
        self.share_sum.len()
    }

    /// The share of attempted deals that could be used, in `0..=1`.
    ///
    /// One when every draw worked, which is the ordinary case. It falls when
    /// seats compete for scarce cards -- several hands all wanting a five
    /// with only two left -- and a low figure is worth showing a caller,
    /// because it is the difference between an answer arriving and a
    /// calculator that appears to have stopped.
    ///
    /// It says less than it used to about how hard a spot was. A request the
    /// sampler decided to weight rather than reject hardly throws a deal away
    /// at all, so this comes back near one however fiercely the seats are
    /// competing. What such a run cost is in
    /// [`effective_samples`](Self::effective_samples) instead, and
    /// [`EquityRequest::is_weighted`](crate::odds::EquityRequest::is_weighted)
    /// says which of the two numbers to read.
    pub fn acceptance(&self) -> f64 {
        if self.attempts == 0 {
            return 1.0;
        }
        self.samples as f64 / self.attempts as f64
    }

    /// Folds `other` into this one.
    ///
    /// The merged result is exact only when both parts were, since mixing a
    /// sample into an enumeration makes the whole thing an estimate. A part
    /// with no deals in it is neither, so it does not vote -- which is what
    /// lets a caller start a loop of its own from [`empty`](Self::empty).
    pub fn merge(&mut self, other: &ChunkResult) {
        debug_assert_eq!(self.seats(), other.seats(), "chunks must cover the same seats");
        if self.samples == 0 {
            self.exact = other.exact;
        } else if other.samples > 0 {
            self.exact = self.exact && other.exact;
        }
        self.samples += other.samples;
        self.attempts += other.attempts;
        self.weight_sum += other.weight_sum;
        self.weight_square_sum += other.weight_square_sum;
        for seat in 0..self.seats() {
            self.share_sum[seat] += other.share_sum[seat];
            self.share_square_sum[seat] += other.share_square_sum[seat];
            self.square_weight_share_sum[seat] += other.square_weight_share_sum[seat];
            self.win_count[seat] += other.win_count[seat];
            self.tie_count[seat] += other.tie_count[seat];
            self.scoop_count[seat] += other.scoop_count[seat];
            self.low_share_sum[seat] += other.low_share_sum[seat];
        }
    }

    /// Records one deal's outcome, counting it once.
    pub(crate) fn record(&mut self, shares: &[f64], low_shares: &[f64]) {
        self.record_weighted(shares, low_shares, 1.0);
    }

    /// Records one deal's outcome, counting it `weight` times.
    ///
    /// A deal drawn against the live deck is not as likely as one drawn
    /// against the whole deck, and the weight is how much that has to be
    /// undone: it is how many hands the seats had to choose from, so a deal
    /// that was easy to reach counts for less. A weight of one is a deal that
    /// needed no correction, which is every deal on the ordinary path.
    pub(crate) fn record_weighted(&mut self, shares: &[f64], low_shares: &[f64], weight: f64) {
        debug_assert!(weight > 0.0 && weight.is_finite(), "a deal's weight must be a positive number");
        self.samples += 1;
        self.weight_sum += weight;
        self.weight_square_sum += weight * weight;
        let square = weight * weight;

        for seat in 0..self.seats() {
            let share = shares[seat];
            self.share_sum[seat] += weight * share;
            self.share_square_sum[seat] += square * share * share;
            self.square_weight_share_sum[seat] += square * share;
            self.low_share_sum[seat] += weight * low_shares[seat];
            if share >= 1.0 - f64::EPSILON {
                self.win_count[seat] += weight;
                self.scoop_count[seat] += weight;
            } else if share > 0.0 {
                self.tie_count[seat] += weight;
            }
        }
    }

    /// How many independent deals these weighted deals are worth.
    ///
    /// Equal to [`samples`](Self::samples) when nothing was weighted. Below
    /// it when the weights are uneven, because a pile in which a few deals
    /// carry most of the total says less than its count suggests. This is
    /// what the error bar is really divided by.
    pub fn effective_samples(&self) -> f64 {
        if self.weight_square_sum <= 0.0 {
            return 0.0;
        }
        self.weight_sum * self.weight_sum / self.weight_square_sum
    }

    /// The finished numbers, one per seat.
    pub fn equities(&self) -> Vec<PlayerEquity> {
        (0..self.seats())
            .map(|seat| {
                let total = if self.weight_sum > 0.0 {
                    self.weight_sum
                } else {
                    self.samples.max(1) as f64
                };
                let equity = self.share_sum[seat] / total;
                // The standard error of a weighted mean, which is the spread
                // of the deals about it divided by the total weight:
                //
                //     sqrt( sum w^2 (x - mean)^2 ) / sum w
                //
                // Expanded so it can be read off the running sums. With every
                // weight at one this is `sum (x - mean)^2 / n^2`, the plain
                // `variance / n` it has always been -- the ordinary path does
                // not merely agree with this, it computes the same thing.
                // Exactly zero when the deals were enumerated, not sampled.
                let std_error = if self.exact || self.samples < 2 {
                    0.0
                } else {
                    let spread = self.share_square_sum[seat]
                        - 2.0 * equity * self.square_weight_share_sum[seat]
                        + equity * equity * self.weight_square_sum;
                    spread.max(0.0).sqrt() / total
                };
                PlayerEquity {
                    equity,
                    win: self.win_count[seat] / total,
                    tie: self.tie_count[seat] / total,
                    low_equity: self.low_share_sum[seat] / total,
                    scoop: self.scoop_count[seat] / total,
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
