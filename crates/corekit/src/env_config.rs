use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvError {
    errors: Vec<EnvVarError>,
}

impl EnvError {
    pub fn new(errors: Vec<EnvVarError>) -> Self {
        Self { errors }
    }

    pub fn errors(&self) -> &[EnvVarError] {
        &self.errors
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.errors.iter().map(EnvVarError::name).collect::<Vec<_>>().join(", ");
        write!(formatter, "failed to load EnvConfig: {names}")
    }
}

impl std::error::Error for EnvError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarError {
    name: String,
    kind: EnvErrorKind,
}

impl EnvVarError {
    pub fn missing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnvErrorKind::Missing,
        }
    }

    pub fn invalid(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnvErrorKind::Invalid,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> EnvErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvErrorKind {
    Missing,
    Invalid,
}
