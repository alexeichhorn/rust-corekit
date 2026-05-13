//@check-pass
use corekit::{retry, timeout, FromTimeout, Retryable};

#[derive(Debug)]
enum Error {
    Timeout,
}

impl FromTimeout for Error {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[retry(max_retries = 3, initial_delay = "500ms", jitter = false)]
#[timeout("1s")]
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {}
