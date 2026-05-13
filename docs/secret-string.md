# SecretString

`SecretString` stores secrets without printing them accidentally.

```rust
use corekit::prelude::*;

let secret = SecretString::from("sk-test");

assert_eq!(secret.expose(), "sk-test");
assert_eq!(format!("{secret:?}"), "SecretString([redacted])");
assert_eq!(secret.to_string(), "[redacted]");
```

Use `.expose()` when a secret must be passed to another API.

```rust
let openai_key = env.OPENAI_API_KEY.expose();
```
