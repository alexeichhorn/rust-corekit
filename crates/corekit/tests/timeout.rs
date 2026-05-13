use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use corekit::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeoutTestError {
    Timeout,
    Inner,
}

impl FromTimeout for TimeoutTestError {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

impl Retryable for TimeoutTestError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout)
    }
}

static RETRY_TIMEOUT_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

#[timeout("2s")]
async fn fast_success() -> Result<&'static str, TimeoutTestError> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok("ok")
}

#[timeout("1s")]
async fn slow_success_times_out() -> Result<&'static str, TimeoutTestError> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok("late")
}

#[timeout("2s")]
async fn inner_error_passes_through() -> Result<&'static str, TimeoutTestError> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Err(TimeoutTestError::Inner)
}

#[timeout("500ms")]
async fn millisecond_timeout() -> Result<&'static str, TimeoutTestError> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok("late")
}

#[retry(max_retries = 2, initial_delay = "0ms", max_delay = "0ms", jitter = false)]
#[timeout("1s")]
async fn retry_wraps_per_attempt_timeout() -> Result<&'static str, TimeoutTestError> {
    RETRY_TIMEOUT_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok("late")
}

#[tokio::test(start_paused = true)]
async fn timeout_passes_through_success_before_deadline() {
    let result = fast_success().await;

    assert_eq!(result, Ok("ok"));
}

#[tokio::test(start_paused = true)]
async fn timeout_maps_elapsed_deadline_to_error_type() {
    let result = slow_success_times_out().await;

    assert_eq!(result, Err(TimeoutTestError::Timeout));
}

#[tokio::test(start_paused = true)]
async fn timeout_passes_through_inner_errors_before_deadline() {
    let result = inner_error_passes_through().await;

    assert_eq!(result, Err(TimeoutTestError::Inner));
}

#[tokio::test(start_paused = true)]
async fn timeout_accepts_millisecond_duration() {
    let result = millisecond_timeout().await;

    assert_eq!(result, Err(TimeoutTestError::Timeout));
}

#[tokio::test(start_paused = true)]
async fn retry_and_timeout_time_out_each_attempt_independently() {
    RETRY_TIMEOUT_ATTEMPTS.store(0, Ordering::SeqCst);

    let result = retry_wraps_per_attempt_timeout().await;

    assert_eq!(result, Err(TimeoutTestError::Timeout));
    assert_eq!(RETRY_TIMEOUT_ATTEMPTS.load(Ordering::SeqCst), 3);
}
