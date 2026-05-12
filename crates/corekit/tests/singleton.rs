use std::sync::atomic::{AtomicUsize, Ordering};

use corekit::singleton;

static CONSTRUCTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static DEFAULT_CALLS: AtomicUsize = AtomicUsize::new(0);
static CUSTOM_INIT_CALLS: AtomicUsize = AtomicUsize::new(0);

#[singleton]
struct RuntimeService {
    value: usize,
}

#[singleton(default)]
struct DefaultRuntimeService {
    value: usize,
}

impl Default for DefaultRuntimeService {
    fn default() -> Self {
        let previous_calls = DEFAULT_CALLS.fetch_add(1, Ordering::SeqCst);
        Self { value: previous_calls + 1 }
    }
}

impl DefaultRuntimeService {
    fn value(&self) -> usize {
        self.value
    }
}

#[singleton(init = CustomInitRuntimeService::build)]
struct CustomInitRuntimeService {
    value: usize,
}

impl CustomInitRuntimeService {
    fn build() -> Self {
        let previous_calls = CUSTOM_INIT_CALLS.fetch_add(1, Ordering::SeqCst);
        Self { value: previous_calls + 1 }
    }

    fn value(&self) -> usize {
        self.value
    }
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

#[test]
fn default_singleton_uses_default_once() {
    let first = DefaultRuntimeService::shared();
    let second = DefaultRuntimeService::shared();

    assert!(std::ptr::eq(first, second));
    assert_eq!(first.value(), 1);
    assert_eq!(DEFAULT_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn custom_init_singleton_uses_configured_constructor_once() {
    let first = CustomInitRuntimeService::shared();
    let second = CustomInitRuntimeService::shared();

    assert!(std::ptr::eq(first, second));
    assert_eq!(first.value(), 1);
    assert_eq!(CUSTOM_INIT_CALLS.load(Ordering::SeqCst), 1);
}
