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

## Supported Types

- `String`
- `bool`
- integer and float types
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
