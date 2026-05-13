#![deny(deprecated)]

use corekit::{retry, Retryable};

#[derive(Debug)]
struct Error;

impl Retryable for Error {
    fn is_retryable(&self) -> bool {
        true
    }
}

#[deprecated]
#[retry(max_retries = 0)]
async fn call() -> Result<(), Error> {
    Ok(())
}

fn main() {
    let _future = call();
    //~^ ERROR: use of deprecated function `call`
}
