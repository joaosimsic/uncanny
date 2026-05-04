use std::time::{Duration, Instant};

pub struct Hysteresis<T: PartialEq + Clone> {
    committed: T,
    pending: Option<(T, Instant)>,
    min_hold: Duration,
}

impl<T: PartialEq + Clone> Hysteresis<T> {
    pub fn new(initial: T, min_hold: Duration) -> Self {
        Self {
            committed: initial,
            pending: None,
            min_hold,
        }
    }

    pub fn committed(&self) -> &T {
        &self.committed
    }

    pub fn update(&mut self, candidate: T, now: Instant) -> T {
        if candidate == self.committed {
            self.pending = None;
            return self.committed.clone();
        }

        match &self.pending {
            Some((pending_val, since)) if *pending_val == candidate => {
                if now.duration_since(*since) >= self.min_hold {
                    self.committed = candidate;
                    self.pending = None;
                }
            }
            _ => {
                self.pending = Some((candidate, now));
            }
        }

        self.committed.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn same_value_returns_committed() {
        let mut h = Hysteresis::new(0i32, Duration::from_millis(300));
        let epoch = Instant::now();
        assert_eq!(h.update(0, epoch), 0);
    }

    #[test]
    fn new_value_held_back_before_min_hold() {
        let mut h = Hysteresis::new(0i32, Duration::from_millis(300));
        let epoch = Instant::now();
        assert_eq!(h.update(1, epoch + Duration::from_millis(100)), 0);
        assert_eq!(h.update(1, epoch + Duration::from_millis(200)), 0);
    }

    #[test]
    fn new_value_committed_after_min_hold() {
        let mut h = Hysteresis::new(0i32, Duration::from_millis(300));
        let epoch = Instant::now();
        h.update(1, epoch);
        let out = h.update(1, epoch + Duration::from_millis(300));
        assert_eq!(out, 1);
    }

    #[test]
    fn flicker_resets_timer() {
        let mut h = Hysteresis::new(0i32, Duration::from_millis(300));
        let epoch = Instant::now();
        h.update(1, epoch);
        h.update(0, epoch + Duration::from_millis(200));
        let out = h.update(1, epoch + Duration::from_millis(250));
        assert_eq!(out, 0, "should still be committed to 0 since timer reset");
    }
}
