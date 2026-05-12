//! Small Rust ergonomics library for backend projects.

mod env_config;
mod secret_string;

pub use corekit_macros::*;
pub use env_config::{EnvError, EnvErrorKind, EnvVarError};
pub use secret_string::SecretString;

pub mod prelude {
    pub use crate::*;
}
