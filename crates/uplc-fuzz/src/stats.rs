use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

/// Thread-safe live statistics.
pub struct Stats {
    pub iterations: AtomicU64,
    pub successes: AtomicU64,
    pub errors: AtomicU64,
    pub divergences: AtomicU64,
    pub panics: AtomicU64,
    pub corpus_size: AtomicU64,
    pub start_time: Instant,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            iterations: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            divergences: AtomicU64::new(0),
            panics: AtomicU64::new(0),
            corpus_size: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn print_summary(&self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let iters = self.iterations.load(Ordering::Relaxed);
        let rate = if elapsed > 0.0 {
            iters as f64 / elapsed
        } else {
            0.0
        };

        eprintln!(
            "[{:.1}s] {:>10} programs | {:>8.0}/s | {:>6} ok | {:>6} err | {:>4} diverge | {:>3} panic | corpus: {}",
            elapsed,
            iters,
            rate,
            self.successes.load(Ordering::Relaxed),
            self.errors.load(Ordering::Relaxed),
            self.divergences.load(Ordering::Relaxed),
            self.panics.load(Ordering::Relaxed),
            self.corpus_size.load(Ordering::Relaxed),
        );
    }
}
