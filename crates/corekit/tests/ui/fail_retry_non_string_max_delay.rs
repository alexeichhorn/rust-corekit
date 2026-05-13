use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_delay = 180)]
//~^ ERROR: `#[retry(max_delay = ...)]` expects a string literal
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
