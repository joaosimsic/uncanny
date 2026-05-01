use arc_swap::ArcSwap;
use std::sync::Arc;

pub struct Slot<T: Clone> {
    inner: ArcSwap<T>,
}

impl<T: Clone> Slot<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: ArcSwap::from_pointee(initial),
        }
    }

    pub fn update(&self, value: T) {
        self.inner.store(Arc::new(value));
    }

    pub fn load(&self) -> T {
        (**self.inner.load()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_roundtrip() {
        let slot = Slot::new(10);
        assert_eq!(slot.load(), 10);
        slot.update(20);
        assert_eq!(slot.load(), 20);
    }
}
