use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(initial_delay = 1)]
//~^ ERROR: `#[retry(initial_delay = ...)]` expects a string literal
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
