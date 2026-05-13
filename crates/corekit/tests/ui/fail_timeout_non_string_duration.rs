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

#[timeout(1)]
//~^ ERROR: `#[timeout(...)]` expects a string literal
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
