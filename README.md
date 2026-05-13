# corekit

Small Rust ergonomics library for backend projects.

## Usage

Import the common API from the prelude:

```rust
use corekit::prelude::*;
```

## EnvConfig

Use `#[env_config]` for typed environment config. Add `global = env` if you want global field-style access.

```rust
use corekit::prelude::*;

#[env_config(global = env)]
pub struct Env {
    pub DATABASE_URL: String,
    pub OPENAI_API_KEY: SecretString,
    pub DEV_MODE: bool,

    #[env(default = 10)]
    pub WORKER_COUNT: u16,

    pub SENTRY_DSN: Option<String>,
}

let database_url = env.DATABASE_URL.as_str();
let openai_key = env.OPENAI_API_KEY.expose();
let worker_count = env.WORKER_COUNT;
```

The explicit derive form is still available when preferred:

```rust
#[derive(EnvConfig)]
#[env_config(global = env)]
pub struct Env {
    pub database_url: String,
}
```

Example `.env`:

```dotenv
DATABASE_URL=postgres://localhost/app
OPENAI_API_KEY=sk-test
DEV_MODE=true
WORKER_COUNT=4
```

`Env::load()` is also generated for tests and tools:

```rust
let env_value = Env::load()?;
```

Behavior:

- `.env` is loaded automatically before reading `std::env`.
- Process env vars win over values from `.env`.
- Missing required vars fail.
- Invalid values fail.
- Errors are collected together.
- First access to the generated global lazily loads the env config.
- Failed global loading panics with the collected env errors.

Dotenv loading can be customized:

```rust
#[env_config(global = env, dotenv = ".env.test")]
pub struct TestEnv {
    pub DATABASE_URL: String,
}

#[env_config(dotenv = false)]
pub struct ProcessOnlyEnv {
    pub DATABASE_URL: String,
}
```

Field names:

```rust
pub database_url: String, // reads DATABASE_URL
pub DATABASE_URL: String, // reads DATABASE_URL

#[env(name = "DATABASE_URL")]
pub db_url: String,
```

Defaults:

```rust
#[env(default = 90)]
pub video_max_duration_s: u64,

#[env(default = false)]
pub dev_mode: bool,

#[env(default = "info")]
pub log_level: String,
```

Optional values:

```rust
pub sentry_dsn: Option<String>,

#[env(optional)]
pub redis_url: Option<String>,
```

Supported value types:

- `String`
- `bool`
- integer and float types
- `Option<T>`
- custom types implementing `FromStr`
- `SecretString`

## SecretString

Use `SecretString` for secrets that should not be printed accidentally.

```rust
let secret = SecretString::from("sk-test");

assert_eq!(secret.expose(), "sk-test");
assert_eq!(format!("{secret:?}"), "SecretString([redacted])");
assert_eq!(secret.to_string(), "[redacted]");
```

## Singleton

Use `#[singleton]` to generate a process-wide `shared()` accessor backed by `std::sync::LazyLock`.

```rust
use corekit::prelude::*;
use std::sync::RwLock;

#[singleton]
pub struct UserService {
    cache: RwLock<Vec<String>>,
}

impl UserService {
    fn new() -> Self {
        Self {
            cache: RwLock::new(Vec::new()),
        }
    }
}

let service: &'static UserService = UserService::shared();
```

Supported init forms:

```rust
#[singleton]                         // Type::new()
#[singleton(default)]                // Type::default()
#[singleton(init = UserService::new)] // custom zero-arg constructor
```

For mutable singleton state, use safe interior mutability like `Mutex`, `RwLock`, or atomics. Do not hold a blocking lock guard across `.await`.
