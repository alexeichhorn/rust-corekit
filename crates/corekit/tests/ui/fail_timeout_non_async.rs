use corekit::{timeout, FromTimeout};

#[derive(Debug)]
enum Error {
    Timeout,
}

impl FromTimeout for Error {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

#[timeout("1s")]
//~^ ERROR: `#[timeout]` can only be used on async functions
fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
