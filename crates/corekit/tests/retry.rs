use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use corekit::prelude::*;
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestError {
    Retryable,
    Fatal,
}

impl Retryable for TestError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Retryable)
    }
}

static IMMEDIATE_SUCCESS_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static RETRY_THEN_SUCCESS_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static NON_RETRYABLE_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static EXHAUSTED_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static ZERO_RETRY_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static BACKOFF_ATTEMPTS: LazyLock<Mutex<Vec<Instant>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static CAPPED_BACKOFF_ATTEMPTS: LazyLock<Mutex<Vec<Instant>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static ZERO_RETRY_WITH_DELAY_ATTEMPTS: LazyLock<Mutex<Vec<Instant>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[retry]
async fn immediate_success() -> Result<&'static str, TestError> {
    IMMEDIATE_SUCCESS_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    Ok("ok")
}

#[retry(max_retries = 3)]
async fn retry_then_success() -> Result<&'static str, TestError> {
    let attempt = RETRY_THEN_SUCCESS_ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;

    if attempt < 3 {
        return Err(TestError::Retryable);
    }

    Ok("ok")
}

#[retry(max_retries = 3)]
async fn non_retryable_failure() -> Result<&'static str, TestError> {
    NON_RETRYABLE_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    Err(TestError::Fatal)
}

#[retry(max_retries = 3)]
async fn exhausted_retryable_failure() -> Result<&'static str, TestError> {
    EXHAUSTED_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    Err(TestError::Retryable)
}

#[retry(max_retries = 0)]
async fn zero_retry_failure() -> Result<&'static str, TestError> {
    ZERO_RETRY_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    Err(TestError::Retryable)
}

#[retry(max_retries = 3, initial_delay = "1s", exponential_base = 2, max_delay = "10s", jitter = false)]
async fn retry_with_backoff() -> Result<&'static str, TestError> {
    BACKOFF_ATTEMPTS.lock().unwrap().push(Instant::now());
    Err(TestError::Retryable)
}

#[retry(max_retries = 4, initial_delay = "1s", exponential_base = 3, max_delay = "5s", jitter = false)]
async fn retry_caps_backoff_at_max_delay() -> Result<&'static str, TestError> {
    CAPPED_BACKOFF_ATTEMPTS.lock().unwrap().push(Instant::now());
    Err(TestError::Retryable)
}

#[retry(max_retries = 0, initial_delay = "1s", exponential_base = 2, max_delay = "10s", jitter = false)]
async fn zero_retry_with_delay() -> Result<&'static str, TestError> {
    ZERO_RETRY_WITH_DELAY_ATTEMPTS.lock().unwrap().push(Instant::now());
    Err(TestError::Retryable)
}

#[tokio::test]
async fn retry_defaults_return_success_without_extra_attempts() {
    IMMEDIATE_SUCCESS_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = immediate_success().await;

    assert_eq!(result, Ok("ok"));
    assert_eq!(IMMEDIATE_SUCCESS_ATTEMPTS.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn retry_retries_retryable_errors_until_success() {
    RETRY_THEN_SUCCESS_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = retry_then_success().await;

    assert_eq!(result, Ok("ok"));
    assert_eq!(RETRY_THEN_SUCCESS_ATTEMPTS.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn retry_stops_on_non_retryable_error() {
    NON_RETRYABLE_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = non_retryable_failure().await;

    assert_eq!(result, Err(TestError::Fatal));
    assert_eq!(NON_RETRYABLE_ATTEMPTS.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn retry_returns_final_error_after_retries_are_exhausted() {
    EXHAUSTED_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = exhausted_retryable_failure().await;

    assert_eq!(result, Err(TestError::Retryable));
    assert_eq!(EXHAUSTED_ATTEMPTS.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn zero_max_retries_means_only_the_initial_attempt_runs() {
    ZERO_RETRY_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = zero_retry_failure().await;

    assert_eq!(result, Err(TestError::Retryable));
    assert_eq!(ZERO_RETRY_ATTEMPTS.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn retry_waits_with_exponential_backoff_between_attempts() {
    BACKOFF_ATTEMPTS.lock().unwrap().clear();

    let result = retry_with_backoff().await;

    assert_eq!(result, Err(TestError::Retryable));

    let attempts = BACKOFF_ATTEMPTS.lock().unwrap();
    assert_eq!(attempts.len(), 4);
    assert_eq!(attempts[1].duration_since(attempts[0]), Duration::from_secs(1));
    assert_eq!(attempts[2].duration_since(attempts[1]), Duration::from_secs(2));
    assert_eq!(attempts[3].duration_since(attempts[2]), Duration::from_secs(4));
}

#[tokio::test(start_paused = true)]
async fn retry_caps_exponential_backoff_at_max_delay() {
    CAPPED_BACKOFF_ATTEMPTS.lock().unwrap().clear();

    let result = retry_caps_backoff_at_max_delay().await;

    assert_eq!(result, Err(TestError::Retryable));

    let attempts = CAPPED_BACKOFF_ATTEMPTS.lock().unwrap();
    assert_eq!(attempts.len(), 5);
    assert_eq!(attempts[1].duration_since(attempts[0]), Duration::from_secs(1));
    assert_eq!(attempts[2].duration_since(attempts[1]), Duration::from_secs(3));
    assert_eq!(attempts[3].duration_since(attempts[2]), Duration::from_secs(5));
    assert_eq!(attempts[4].duration_since(attempts[3]), Duration::from_secs(5));
}

#[tokio::test(start_paused = true)]
async fn zero_max_retries_with_delay_never_sleeps() {
    ZERO_RETRY_WITH_DELAY_ATTEMPTS.lock().unwrap().clear();
    let started_at = Instant::now();

    let result = zero_retry_with_delay().await;

    assert_eq!(result, Err(TestError::Retryable));

    let attempts = ZERO_RETRY_WITH_DELAY_ATTEMPTS.lock().unwrap();
    assert_eq!(attempts.as_slice(), &[started_at]);
    assert_eq!(Instant::now(), started_at);
}
