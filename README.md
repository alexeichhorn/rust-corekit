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

    #[env(default = 30)]
    pub REQUEST_TIMEOUT_SECONDS: u64,

    #[env(optional)]
    pub SENTRY_DSN: Option<String>,
}

#[singleton]
pub struct UserService {
    http_client: reqwest::Client,
    openai_client: OpenAIClient,
}

impl UserService {
    fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(env.REQUEST_TIMEOUT_SECONDS))
            .build()
            .expect("failed to build reqwest client");
        let openai_client = OpenAIClient::new(http_client.clone(), env.OPENAI_API_KEY.expose());

        Self {
            http_client,
            openai_client,
        }
    }
}
```

Example `.env`:

```dotenv
DATABASE_URL=postgres://localhost/app
OPENAI_API_KEY=sk-test
DEV_MODE=true
REQUEST_TIMEOUT_SECONDS=30
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

## More Docs

- [EnvConfig](docs/env-config.md)
- [Singleton](docs/singleton.md)
- [SecretString](docs/secret-string.md)
- [Retry](docs/retry.md)
