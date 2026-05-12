use corekit::singleton;

#[singleton] //~ ERROR: this function takes 1 argument but 0 arguments were supplied
struct NeedsArgument;

impl NeedsArgument {
    fn new(_name: &str) -> Self {
        Self
    }
}

fn main() {
    let _service = NeedsArgument::shared();
}
