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

## More Docs

- [EnvConfig](docs/env-config.md)
- [Singleton](docs/singleton.md)
- [SecretString](docs/secret-string.md)
