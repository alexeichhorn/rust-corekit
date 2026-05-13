use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(initial_delay = "soon")]
//~^ ERROR: `#[retry(initial_delay = ...)]` expects a duration string like "500ms", "1s", "2m", or "1h"
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
