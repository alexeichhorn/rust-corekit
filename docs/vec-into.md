# VecInto

`VecInto` converts every item in a `Vec` using `Into`.

```rust
use corekit::prelude::*;

let numbers = vec![1_u8, 2, 3];
let widened: Vec<u16> = numbers.vec_into();

assert_eq!(widened, vec![1_u16, 2, 3]);
```

This is most useful when converting collections of internal models into response DTOs.
