//! Small Rust ergonomics library for backend projects.

#[path = "env_config.rs"]
mod env_config_runtime;
mod retry;
mod secret_string;
mod timeout;
mod vec_into;

pub use corekit_macros::*;
pub use env_config_runtime::{EnvError, EnvErrorKind, EnvVarError};
pub use retry::Retryable;
pub use secret_string::SecretString;
pub use timeout::FromTimeout;
pub use vec_into::VecInto;

pub mod prelude {
    pub use crate::*;
}

#[doc(hidden)]
pub mod __private {
    pub use dotenvy;
    pub use tokio;
}
