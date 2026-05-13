//@check-pass
use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_retries = 5, initial_delay = "1s", exponential_base = 2, max_delay = "180s", jitter = true)]
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
