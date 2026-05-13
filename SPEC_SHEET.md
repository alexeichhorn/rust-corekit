# corekit SPEC SHEET

## TL;DR

`corekit` is a small Rust ergonomics library for backend projects.

It should make common backend patterns feel cleaner and more consistent without becoming a full framework.

Initial public features:

- `#[singleton]`
- `#[env_config(...)]`
- `#[retry(...)]`
- `#[timeout(...)]`
- Later: shared LLM layer, exact API open for discussion

Out of scope for now:

- Axum error framework
- database abstraction
- HTTP client wrapper
- typed ID macro
- app context singleton
- generic repository framework

---

# Basic usage

Projects should be able to import the common API like this:

```rust
use corekit::prelude::*;
```

Example:

```rust
use corekit::prelude::*;

#[env_config(global = env)]
pub struct Env {
    pub DATABASE_URL: String,
    pub OPENAI_API_KEY: SecretString,
    pub DEV_MODE: bool,
}

#[singleton]
pub struct UserService;

impl UserService {
    fn new() -> Self {
        Self
    }

    pub async fn create_user(&self, input: CreateUser) -> Result<User, UserError> {
        let database_url = env.DATABASE_URL.as_str();
        let openai_key = env.OPENAI_API_KEY.expose();
        todo!("use input, database_url, and openai_key")
    }
}
```

Usage:

```rust
let user = UserService::shared()
    .create_user(input)
    .await?;
```

---

# Feature: `#[singleton]`

## User-facing goal

Allow service structs to become singleton-style services without writing `LazyLock` boilerplate manually.

```rust
#[singleton]
pub struct UserService {
    cache: RwLock<HashMap<UserId, User>>,
}
```

The user can then call:

```rust
UserService::shared()
    .create_user(input)
    .await?;
```

## Expected behavior

`#[singleton]` generates a `shared()` method:

```rust
impl UserService {
    pub fn shared() -> &'static Self {
        // generated singleton instance
    }
}
```

For v1, the struct must provide:

```rust
fn new() -> Self
```

The constructor may be private.

## Implementation note

Internally this should use `std::sync::LazyLock`.

Roughly:

```rust
static INSTANCE: LazyLock<UserService> = LazyLock::new(UserService::new);
```

## Important rule

If the singleton has mutable state, the user must use safe interior mutability:

- `std::sync::Mutex`
- `std::sync::RwLock`
- `tokio::sync::Mutex`
- `tokio::sync::RwLock`
- atomics

Do not hold a blocking lock guard across `.await`.

---

# Feature: `#[env_config(...)]`

## User-facing goal

Replace Python-style `env.py` files with one typed Rust struct that can be accessed globally without passing config through every service constructor.

Primary API:

```rust
#[env_config(global = env)]
pub struct Env {
    pub DATABASE_URL: String,
    pub REDIS_PORT: u16,
    pub DEV_MODE: bool,
    pub OPENAI_API_KEY: SecretString,
}
```

The attribute macro owns the struct item, so uppercase env-style field names should not require a separate `#[allow(non_snake_case)]`.

For explicit derive-based usage, `#[derive(EnvConfig)]` remains supported:

```rust
#[derive(EnvConfig)]
#[env_config(global = env)]
pub struct Env {
    pub database_url: String,
}
```

Normal app usage:

```rust
let db_url = env.DATABASE_URL.as_str();
let openai_key = env.OPENAI_API_KEY.expose();

if env.DEV_MODE {
    // development behavior
}
```

For tests/tools, the macro also generates an explicit loader:

```rust
let env_value = Env::load()?;
```

## Requirements

- With `#[env_config(global = env)]`, first access to `env.FIELD` lazily loads and validates all env vars.
- If global lazy loading fails, it panics with all collected env errors.
- `Env::load()` returns `Result<Self, EnvError>` and does not touch the global instance.
- Required env vars fail if missing.
- Invalid values fail.
- Errors should be collected and shown together.
- Fields are type-safe.
- Defaults are simple.
- Optional values are simple.
- Secrets should not be accidentally printed in debug logs.

## Field mapping

By default, snake-case field names map to screaming snake case:

```rust
pub db_url: String,
pub redis_port: u16,
```

maps to:

```text
DB_URL
REDIS_PORT
```

Uppercase field names map to themselves, so no double definition is needed:

```rust
#[env_config(global = env)]
pub struct Env {
    pub DATABASE_URL: String,
    pub OPENAI_API_KEY: SecretString,
}
```

maps to:

```text
DATABASE_URL
OPENAI_API_KEY
```

Custom env name:

```rust
#[env(name = "DATABASE_URL")]
pub db_url: String,
```

## Defaults

```rust
#[env(default = 90)]
pub video_max_duration_s: u64,
```

```rust
#[env(default = false)]
pub dev_mode: bool,
```

If the env var is missing, the default is used.

If the env var is present but invalid, loading fails.

## Optional values

```rust
#[env(optional)]
pub sentry_dsn: Option<String>,
```

`Option<T>` should also imply optional.

## Supported types for v1

- `String`
- `bool`
- integer types
- float types
- `Option<T>`
- custom types implementing `FromStr`
- `SecretString`

Later possible additions:

- `Vec<String>`
- JSON values
- custom structs from JSON env vars

## Secret values

Add a `SecretString` type.

```rust
pub OPENAI_API_KEY: SecretString,
```

`Debug` output should be redacted.

Access should be explicit:

```rust
env.OPENAI_API_KEY.expose()
```

## Implementation note

The macro should generate an inherent loader:

```rust
impl Env {
    pub fn load() -> Result<Self, EnvError> {
        todo!()
    }
}
```

When `#[env_config(global = env)]` is present, it should also generate a lazy global value:

```rust
#[allow(non_upper_case_globals)]
pub static env: std::sync::LazyLock<Env> = std::sync::LazyLock::new(|| {
    Env::load().expect("failed to load EnvConfig")
});
```

Simple v1 dotenv behavior is enough:

```rust
dotenvy::dotenv().ok();
```

Then read from `std::env`.

---

# Feature: `#[retry(...)]`

## User-facing goal

Provide a Python-decorator-like retry wrapper for async functions.

```rust
#[retry(max_retries = 3, initial_delay = "500ms", max_delay = "10s")]
pub async fn call_external_api(input: Input) -> Result<Output, ExternalApiError> {
    todo!()
}
```

## Requirements

- Works on async functions.
- Function must return `Result<T, E>`.
- Retries only retryable errors.
- Supports exponential backoff.
- Supports max delay.
- Optional jitter can be added later.

## Retryable errors

Error types decide whether they are retryable:

```rust
pub trait Retryable {
    fn is_retryable(&self) -> bool;
}
```

Example:

```rust
impl Retryable for ExternalApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout | Self::RateLimited | Self::ServerError)
    }
}
```

## Syntax

```rust
#[retry]
```

```rust
#[retry(max_retries = 3)]
```

```rust
#[retry(max_retries = 3, initial_delay = "500ms")]
```

```rust
#[retry(max_retries = 5, initial_delay = "1s", max_delay = "30s")]
```

`max_retries = 3` means:

```text
1 initial try + 3 retries = up to 4 total attempts
```

## Defaults

When no arguments are supplied, use the same defaults as the CareerArc backend retry wrapper:

- `max_retries = 20`
- `initial_delay = "1s"`
- `exponential_base = 2`
- `max_delay = "180s"`
- `jitter = true`

## Implementation note

The macro wraps the original function body in a retry loop.

It should use the same duration syntax as `#[timeout]`:

```text
100ms
500ms
1s
10s
2m
1h
```

---

# Feature: `#[timeout(...)]`

## User-facing goal

Provide a Python-decorator-like timeout wrapper for async functions.

```rust
#[timeout("30s")]
pub async fn scrape_page(url: Url) -> Result<Page, ScrapeError> {
    todo!()
}
```

## Requirements

- Works on async functions.
- Function must return `Result<T, E>`.
- Uses `tokio::time::timeout`.
- Maps timeout into the function's error type.

## Timeout errors

Error types define how a timeout is represented:

```rust
pub trait FromTimeout {
    fn from_timeout() -> Self;
}
```

Example:

```rust
impl FromTimeout for ScrapeError {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}
```

## Syntax

```rust
#[timeout("500ms")]
```

```rust
#[timeout("10s")]
```

```rust
#[timeout("2m")]
```

## Usage with `#[retry]`

Both should work together:

```rust
#[retry(max_retries = 3, initial_delay = "500ms")]
#[timeout("10s")]
pub async fn call_external_api() -> Result<Response, ExternalApiError> {
    todo!()
}
```

Expected meaning:

```text
retry the function, with each attempt individually timed out
```

So the error type may implement both:

```rust
impl FromTimeout for ExternalApiError {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

impl Retryable for ExternalApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout | Self::RateLimited | Self::ServerError)
    }
}
```

---

# Later feature: LLM layer

We want a shared LLM abstraction eventually.

Reasons:

- prompt templates
- typed outputs
- OpenAI execution
- retry behavior
- timeout behavior
- logging/cost tracking
- testing/mocking support

The exact API and structure are intentionally open for discussion.

Do not design this in v1.

---

# Non-goals

`corekit` should not become:

- a web framework
- an ORM
- a generic repository framework
- an Axum abstraction layer
- a large dependency-heavy kitchen sink

It should stay a small ergonomics layer for code we repeatedly write across backend projects.
