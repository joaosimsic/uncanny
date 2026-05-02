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

    #[must_use]
    pub fn load(&self) -> T {
        (**self.inner.load()).clone()
    }

    #[must_use]
    pub fn load_arc(&self) -> Arc<T> {
        self.inner.load_full()
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

    #[test]
    fn load_arc_shares_pointer() {
        let slot = Slot::new(vec![1, 2, 3]);
        let a = slot.load_arc();
        let b = slot.load_arc();
        assert!(Arc::ptr_eq(&a, &b));
        slot.update(vec![4, 5, 6]);
        let c = slot.load_arc();
        assert!(!Arc::ptr_eq(&a, &c));
        assert_eq!(*c, vec![4, 5, 6]);
    }
}
