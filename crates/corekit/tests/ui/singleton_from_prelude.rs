//@check-pass
use corekit::prelude::*;

#[singleton]
struct PreludeService;

impl PreludeService {
    fn new() -> Self {
        Self
    }
}

fn main() {
    let _service = PreludeService::shared();
}
