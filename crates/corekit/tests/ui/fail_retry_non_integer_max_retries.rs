use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_retries = "3")]
//~^ ERROR: `#[retry(max_retries = ...)]` expects a non-negative integer literal
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
