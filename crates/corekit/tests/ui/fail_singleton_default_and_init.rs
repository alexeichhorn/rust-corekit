use corekit::singleton;

#[singleton(default, init = ConflictingInit::new)]
struct ConflictingInit;
//~^ ERROR: `#[singleton]` accepts only one init strategy

impl ConflictingInit {
    fn new() -> Self {
        Self
    }
}

fn main() {}
