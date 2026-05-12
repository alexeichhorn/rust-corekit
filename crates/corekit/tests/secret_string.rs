use corekit::prelude::*;

#[test]
fn exposes_secret_only_through_explicit_method() {
    let secret = SecretString::from("sk-test-secret");
    let exposed: &str = secret.expose();

    assert_eq!(exposed, "sk-test-secret");
}

#[test]
fn debug_and_display_are_redacted() {
    let secret = SecretString::from("sk-test-secret");

    let debug = format!("{secret:?}");
    let display = secret.to_string();

    assert_eq!(debug, "SecretString([redacted])");
    assert_eq!(display, "[redacted]");
    assert!(!debug.contains("sk-test-secret"));
    assert!(!display.contains("sk-test-secret"));
}

#[test]
fn clone_and_equality_use_the_underlying_secret() {
    let first = SecretString::from("sk-test-secret");
    let second = first.clone();
    let different = SecretString::from("sk-other-secret");

    assert_eq!(first, second);
    assert_ne!(first, different);
    assert_eq!(second.expose(), "sk-test-secret");
}

#[test]
fn owned_strings_can_be_wrapped_without_losing_the_value() {
    let raw = String::from("sk-owned-secret");
    let secret = SecretString::from(raw);

    assert_eq!(secret.expose(), "sk-owned-secret");
}
