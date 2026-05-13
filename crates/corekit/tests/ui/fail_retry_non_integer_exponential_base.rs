use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(exponential_base = "2")]
//~^ ERROR: `#[retry(exponential_base = ...)]` expects an integer literal greater than 0
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
