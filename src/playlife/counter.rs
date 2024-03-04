/// Counter struct for various timers.
#[derive(Clone, Copy)]
pub struct Counter(pub usize);

impl Counter {
    /// True if the counter has reached 0.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Reduce the count by 1 if non-zero.
    pub fn decr(self) -> Self {
        if self.0 > 0 {
            Counter(self.0 - 1)
        } else {
            self
        }
    }
}
