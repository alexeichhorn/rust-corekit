use corekit::singleton;

#[singleton]
struct NeedsArgument;
//~^ ERROR: this function takes 1 argument but 0 arguments were supplied

impl NeedsArgument {
    fn new(_name: &str) -> Self {
        Self
    }
}

fn main() {
    let _service = NeedsArgument::shared();
}
