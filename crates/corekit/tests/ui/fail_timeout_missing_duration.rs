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

#[timeout]
//~^ ERROR: `#[timeout(...)]` expects exactly one duration string
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
