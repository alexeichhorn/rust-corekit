use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_delay = "2d")]
//~^ ERROR: `#[retry(max_delay = ...)]` expects a duration string like "500ms", "1s", "2m", or "1h"
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
