# Poem OpenAPI

Enable the `poem-openapi` feature to use helpers that depend on Poem.

```toml
corekit = { version = "0.1", features = ["poem-openapi"] }
```

## Route Groups

Use `route_group!` when a parent feature owns the inherited route prefix and child APIs should only declare their local paths.

```rust
use corekit::route_group;

pub fn api() -> impl poem_openapi::OpenApi {
    route_group!("/todos", (TodoRoutes, TodoActivityRoutes))
}
```

`route_group!` prefixes both the OpenAPI metadata paths and the Poem route table paths. Root paths are replaced by the prefix, so an API route declared as `/` under `"/todos"` becomes `"/todos"`, not `"/todos/"`.
