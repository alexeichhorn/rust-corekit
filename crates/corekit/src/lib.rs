//! Small Rust ergonomics library for backend projects.

mod secret_string;

pub use corekit_macros::*;
pub use secret_string::SecretString;

pub mod prelude {
    pub use crate::*;
}
