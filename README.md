# corekit

Small Rust ergonomics library for backend projects.

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
