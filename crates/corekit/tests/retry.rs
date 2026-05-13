use std::sync::atomic::{AtomicUsize, Ordering};

use corekit::prelude::*;

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

#[tokio::test]
async fn retry_defaults_return_success_without_extra_attempts() {
    IMMEDIATE_SUCCESS_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = immediate_success().await;

    assert_eq!(result, Ok("ok"));
    assert_eq!(IMMEDIATE_SUCCESS_ATTEMPTS.load(Ordering::SeqCst), 1);
}

#[tokio::test]
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

#[tokio::test]
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
