use corekit::singleton;

#[singleton(init = InitNeedsArgument::build)]
struct InitNeedsArgument;
//~^ ERROR: this function takes 1 argument but 0 arguments were supplied

impl InitNeedsArgument {
    fn build(_name: &str) -> Self {
        Self
    }
}

fn main() {
    let _service = InitNeedsArgument::shared();
}
