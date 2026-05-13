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

#[timeout("2d")]
//~^ ERROR: `#[timeout(...)]` expects a duration string like "500ms", "1s", "2m", or "1h"
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
