use std::sync::atomic::{AtomicUsize, Ordering};

use corekit::singleton;

static CONSTRUCTOR_CALLS: AtomicUsize = AtomicUsize::new(0);

#[singleton]
struct RuntimeService {
    value: usize,
}

impl RuntimeService {
    fn new() -> Self {
        let previous_calls = CONSTRUCTOR_CALLS.fetch_add(1, Ordering::SeqCst);
        Self { value: previous_calls + 1 }
    }

    fn value(&self) -> usize {
        self.value
    }
}

#[test]
fn shared_returns_a_static_reference() {
    let service: &'static RuntimeService = RuntimeService::shared();

    assert_eq!(service.value(), 1);
}

#[test]
fn shared_reuses_the_same_instance() {
    let first = RuntimeService::shared();
    let second = RuntimeService::shared();

    assert!(std::ptr::eq(first, second));
    assert_eq!(CONSTRUCTOR_CALLS.load(Ordering::SeqCst), 1);
}
