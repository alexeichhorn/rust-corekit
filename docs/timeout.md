# Timeout

`#[timeout]` wraps an async `Result` function with `tokio::time::timeout`.

## Example

```rust
use corekit::prelude::*;

#[derive(Debug)]
enum ApiError {
    Timeout,
    InvalidRequest,
}

impl FromTimeout for ApiError {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

#[timeout("10s")]
async fn call_api() -> Result<String, ApiError> {
    Ok("ok".to_owned())
}
```

If the function finishes before the deadline, the original `Result` is returned unchanged. If the deadline elapses, the macro returns `Err(FromTimeout::from_timeout())`.

## Durations

Supported duration units:

```rust
#[timeout("500ms")]
#[timeout("10s")]
#[timeout("2m")]
#[timeout("1h")]
```

## FromTimeout

The error type must implement `FromTimeout`:

```rust
impl FromTimeout for ApiError {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}
```

## With Retry

Put `#[retry]` above `#[timeout]` when each retry attempt should have its own timeout:

```rust
#[retry(max_retries = 3, initial_delay = "500ms", jitter = false)]
#[timeout("10s")]
async fn call_api() -> Result<String, ApiError> {
    Ok("ok".to_owned())
}
```

If the timeout error is retryable, implement both traits:

```rust
impl Retryable for ApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout)
    }
}
```
