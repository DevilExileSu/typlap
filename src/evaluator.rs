use std::time::{Duration, Instant};
pub enum EvalResult {
    Snap(f64, f64),
    Done(Duration, f64, f64, f64),
}


pub struct Evaluator {
    pub total_chars_typed: usize,
    pub total_char_errors: usize,
    pub final_chars_typed_correctly: usize,
    pub final_uncorrected_errors: usize,
    pub start_at: Instant,
    pub pause_at: Instant,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            total_chars_typed: 0,
            total_char_errors: 0,
            final_chars_typed_correctly: 0,
            final_uncorrected_errors: 0,
            start_at: Instant::now(),
            pause_at: Instant::now(),
        }
    }
    pub fn reset(&mut self) {
        self.total_char_errors = 0;
        self.total_chars_typed = 0;
        self.final_chars_typed_correctly = 0;
        self.final_uncorrected_errors = 0;
    }
    pub fn accuracy(&self) -> f64 {
        (self.total_chars_typed as isize - self.total_char_errors as isize) as f64
        / self.total_chars_typed as f64
    }

    pub fn real_time_accuracy(&self) -> f64 {
        self.final_chars_typed_correctly  as f64
        / (self.final_chars_typed_correctly + self.final_uncorrected_errors) as f64
    }

    pub fn real_time_wpm(&self, delta: Duration) -> f64 {
        (self.final_chars_typed_correctly as f64 / 5.0 - self.final_uncorrected_errors as f64)
        .max(0.0) as f64
        / (delta.as_secs_f64() / 60.0)
    }

    pub fn wpm(&self, delta: Duration) -> f64 {
        (self.total_chars_typed as f64 / 5.0 - self.total_char_errors as f64)
        .max(0.0) as f64
        / (delta.as_secs_f64() / 60.0)
    }

    pub fn snap(&self, delta: Duration) -> EvalResult {
        EvalResult::Snap(self.real_time_accuracy(), self.real_time_wpm(delta))
    }

    pub fn done(&self, delta: Duration) -> EvalResult {
        EvalResult::Done(delta, self.real_time_accuracy(), self.accuracy(), self.wpm(delta))
    }
}
