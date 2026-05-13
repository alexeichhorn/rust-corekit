use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_retries = 3, max_retries = 4)]
//~^ ERROR: duplicate `#[retry(max_retries = ...)]` argument
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
