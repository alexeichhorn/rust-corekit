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
        let errors = self.errors.iter().map(|error| error.to_string()).collect::<Vec<_>>().join(", ");
        write!(formatter, "failed to load EnvConfig: {errors}")
    }
}

impl std::error::Error for EnvError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarError {
    name: String,
    kind: EnvErrorKind,
    reason: String,
}

impl EnvVarError {
    pub fn missing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnvErrorKind::Missing,
            reason: "missing required env var".to_owned(),
        }
    }

    pub fn invalid(name: impl Into<String>) -> Self {
        Self::invalid_with_reason(name, "invalid value")
    }

    pub fn invalid_not_unicode(name: impl Into<String>) -> Self {
        Self::invalid_with_reason(name, "value is not valid Unicode")
    }

    pub fn invalid_parse(name: impl Into<String>, expected: impl fmt::Display) -> Self {
        Self::invalid_with_reason(name, format!("invalid value, expected {expected}"))
    }

    pub fn invalid_empty(name: impl Into<String>) -> Self {
        Self::invalid_with_reason(name, "must not be empty or whitespace only")
    }

    pub fn invalid_nan(name: impl Into<String>) -> Self {
        Self::invalid_with_reason(name, "must not be NaN")
    }

    pub fn invalid_below_min(name: impl Into<String>, min: impl fmt::Display) -> Self {
        Self::invalid_with_reason(name, format!("must be greater than or equal to {min}"))
    }

    pub fn invalid_above_max(name: impl Into<String>, max: impl fmt::Display) -> Self {
        Self::invalid_with_reason(name, format!("must be less than or equal to {max}"))
    }

    fn invalid_with_reason(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnvErrorKind::Invalid,
            reason: reason.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> EnvErrorKind {
        self.kind
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for EnvVarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.name, self.reason)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvErrorKind {
    Missing,
    Invalid,
}
