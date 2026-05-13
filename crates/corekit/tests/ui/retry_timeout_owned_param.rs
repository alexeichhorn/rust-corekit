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

#[retry(max_retries = 1, initial_delay = "0ms", max_delay = "0ms", jitter = false)]
#[timeout("1s")]
async fn call(input: String) -> Result<String, Error> {
    let _ = input.as_str();
    Err(Error::Timeout)
}

fn main() {}
