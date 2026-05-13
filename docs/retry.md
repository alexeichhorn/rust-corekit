# Retry

`#[retry]` retries async `Result` functions when the returned error says it is retryable.

## Example

```rust
use corekit::prelude::*;

#[derive(Debug)]
enum ApiError {
    RateLimited,
    InvalidRequest,
}

impl Retryable for ApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited)
    }
}

#[retry(max_retries = 3)]
async fn call_api() -> Result<String, ApiError> {
    Ok("ok".to_owned())
}
```

`max_retries = 3` means one initial attempt plus up to three retries.

Full retry policy shape:

```rust
#[retry(
    max_retries = 5,
    initial_delay = "1s",
    exponential_base = 2,
    max_delay = "180s",
    jitter = true,
)]
async fn call_api() -> Result<String, ApiError> {
    Ok("ok".to_owned())
}
```

## Defaults

```rust
#[retry]
async fn call_api() -> Result<String, ApiError> {
    Ok("ok".to_owned())
}
```

Without arguments, `max_retries` defaults to `20`.

The intended default policy matches the CareerArc backend wrapper:

- `max_retries = 20`
- `initial_delay = "1s"`
- `exponential_base = 2`
- `max_delay = "180s"`
- `jitter = true`

## Retryable

The error type must implement `Retryable`:

```rust
impl Retryable for ApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited)
    }
}
```

Non-retryable errors return immediately.

## Current Scope

The current retry loop supports `max_retries` and retries immediately. `initial_delay`, `exponential_base`, `max_delay`, and `jitter` are documented here as the next retry slice, but are not accepted by the macro yet.
