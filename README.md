# corekit

Small Rust ergonomics library for backend projects.

## Usage

```rust
use corekit::prelude::*;
```

## EnvConfig

Define one typed env struct and access it globally from services.
`.env` is loaded automatically before reading process env vars.

```rust
use corekit::prelude::*;

#[env_config(global = env)]
pub struct Env {
    pub DATABASE_URL: String,
    pub OPENAI_API_KEY: SecretString,

    #[env(default = false)]
    pub DEV_MODE: bool,

    pub SENTRY_DSN: Option<String>,
}

#[singleton]
pub struct UserService;

impl UserService {
    fn new() -> Self {
        Self
    }

    pub fn database_url(&self) -> &str {
        env.DATABASE_URL.as_str()
    }

    pub fn openai_key(&self) -> &str {
        env.OPENAI_API_KEY.expose()
    }
}
```

Example `.env`:

```dotenv
DATABASE_URL=postgres://localhost/app
OPENAI_API_KEY=sk-test
DEV_MODE=true
```

## Singleton

Use `#[singleton]` for service-style singletons.

```rust
use corekit::prelude::*;

#[singleton]
pub struct UserService;

impl UserService {
    fn new() -> Self {
        Self
    }
}

let service = UserService::shared();
```

## Retry

Use `#[retry]` on async `Result` functions. The error type decides which failures are retryable.

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

## Timeout

Use `#[timeout]` on async `Result` functions. The error type decides how a timeout is represented.

```rust
use corekit::prelude::*;

#[derive(Debug)]
enum ApiError {
    Timeout,
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

## More Docs

- [EnvConfig](docs/env-config.md)
- [Singleton](docs/singleton.md)
- [SecretString](docs/secret-string.md)
- [Retry](docs/retry.md)
- [Timeout](docs/timeout.md)
