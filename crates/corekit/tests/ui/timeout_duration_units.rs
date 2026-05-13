//@check-pass
use corekit::{timeout, FromTimeout};

#[derive(Debug)]
enum Error {
    Timeout,
}

impl FromTimeout for Error {
    fn from_timeout() -> Self {
        Self::Timeout
    }
}

#[timeout("500ms")]
async fn milliseconds() -> Result<(), Error> {
    Ok(())
}

#[timeout("10s")]
async fn seconds() -> Result<(), Error> {
    Ok(())
}

#[timeout("2m")]
async fn minutes() -> Result<(), Error> {
    Ok(())
}

#[timeout("1h")]
async fn hours() -> Result<(), Error> {
    Ok(())
}

fn main() {}
