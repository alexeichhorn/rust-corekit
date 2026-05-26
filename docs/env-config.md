# EnvConfig

`#[env_config]` creates a typed env loader and can optionally create a lazy global value.

## Recommended Form

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

let database_url = env.DATABASE_URL.as_str();
let openai_key = env.OPENAI_API_KEY.expose();
```

The attribute form automatically allows uppercase field names.

## Loading

The generated global loads on first access. If loading fails, it panics with all collected env errors.

The macro also generates an explicit loader:

```rust
let env_value = Env::load()?;
```

This is useful in tests and tools.

## Dotenv

By default, `.env` is loaded before reading `std::env`.

```dotenv
DATABASE_URL=postgres://localhost/app
OPENAI_API_KEY=sk-test
DEV_MODE=true
```

Process env vars win over values from `.env`.

Custom dotenv file:

```rust
#[env_config(global = env, dotenv = ".env.test")]
pub struct Env {
    pub DATABASE_URL: String,
}
```

Disable dotenv loading:

```rust
#[env_config(global = env, dotenv = false)]
pub struct Env {
    pub DATABASE_URL: String,
}
```

## Field Names

Snake-case Rust fields map to screaming snake case:

```rust
pub database_url: String, // reads DATABASE_URL
```

Uppercase fields map to themselves:

```rust
pub DATABASE_URL: String, // reads DATABASE_URL
```

Custom env name:

```rust
#[env(name = "DATABASE_URL")]
pub db_url: String,
```

## Defaults

If the env var is missing, the default is used.

```rust
#[env(default = 90)]
pub VIDEO_MAX_DURATION_S: u64,

#[env(default = false)]
pub DEV_MODE: bool,

#[env(default = "info")]
pub LOG_LEVEL: String,
```

If the env var is present but invalid, loading fails.

## Optional Values

`Option<T>` is optional automatically:

```rust
pub SENTRY_DSN: Option<String>,
```

The explicit form is also supported:

```rust
#[env(optional)]
pub REDIS_URL: Option<String>,
```

If the env var is present but invalid, loading fails.

## String Rules

String fields keep the exact env value by default, including leading and trailing whitespace.

Use `trim` when the loaded value should be trimmed before it is stored:

```rust
#[env(trim)]
pub PUBLIC_BASE_URL: String,
```

Use `non_empty` when empty or whitespace-only values should be rejected:

```rust
#[env(non_empty)]
pub DATABASE_URL: String,
```

The rules can be combined and also work with `Option<String>` and string defaults:

```rust
#[env(default = "info", trim, non_empty)]
pub LOG_LEVEL: String,

#[env(trim, non_empty)]
pub SENTRY_DSN: Option<String>,
```

`non_empty` checks `value.trim().is_empty()`. Without `trim`, the stored value is still left unchanged.

## Numeric Rules

Integer and float fields can define inclusive bounds:

```rust
#[env(min = 1, max = 60)]
pub REQUEST_TIMEOUT_SECONDS: u64,

#[env(default = 4, min = 1, max = 8)]
pub WORKER_THREADS: usize,

#[env(min = 0.0, max = 1.0)]
pub SAMPLING_RATE: f64,
```

`min` and `max` also work with numeric `Option<T>` fields and numeric defaults. Missing optional values stay `None`.

```rust
#[env(min = 1024)]
pub PORT: Option<u16>,

#[env(default = 30, min = 1, max = 60)]
pub REQUEST_TIMEOUT_SECONDS: u64,
```

Out-of-range values fail loading with an invalid env var error.

## Supported Types

- `String`
- `bool`
- integer and float types, including `usize`
- `Option<T>`
- custom types implementing `FromStr`
- `SecretString`

## Explicit Derive Form

The attribute form is recommended, but the derive form is still available:

```rust
#[derive(EnvConfig)]
#[env_config(global = env)]
pub struct Env {
    pub database_url: String,
}
```

With the derive form, uppercase Rust fields require your own `#[allow(non_snake_case)]`.
