use corekit::singleton;

#[singleton]
enum NotAService {
    //~^ ERROR: `#[singleton]` can only be used on structs
    Unit,
}

fn main() {}
