//@check-pass
use corekit::prelude::*;

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
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
