use corekit::singleton;

#[singleton]
struct GenericService<T> {
    //~^ ERROR: `#[singleton]` does not support generic structs
    value: T,
}

impl<T> GenericService<T> {
    fn new() -> Self {
        todo!()
    }
}

fn main() {}
