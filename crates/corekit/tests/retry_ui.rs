use std::path::PathBuf;

use ui_test::custom_flags::rustfix::RustfixMode;
use ui_test::dependencies::DependencyBuilder;

fn main() -> ui_test::color_eyre::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config = ui_test::Config::rustc("tests/ui");
    config.out_dir = manifest_dir.join("../../target/ui-corekit");
    config.comment_defaults.base().set_custom(
        "dependencies",
        DependencyBuilder {
            crate_manifest_path: PathBuf::from("tests/ui/Cargo.toml"),
            ..DependencyBuilder::default()
        },
    );
    config.comment_defaults.base().set_custom("rustfix", RustfixMode::Disabled);
    config.output_conflict_handling = ui_test::ignore_output_conflict;
    config.filter_files = vec![
        "retry_basic.rs".into(),
        "retry_from_prelude.rs".into(),
        "retry_defaults.rs".into(),
        "retry_full_policy.rs".into(),
        "retry_timeout_owned_param.rs".into(),
        "fail_retry_preserves_outer_deprecated_attr.rs".into(),
        "fail_retry_non_async.rs".into(),
        "fail_retry_unknown_arg.rs".into(),
        "fail_retry_non_integer_max_retries.rs".into(),
        "fail_retry_duplicate_arg.rs".into(),
        "fail_retry_invalid_initial_delay.rs".into(),
        "fail_retry_invalid_max_delay.rs".into(),
        "fail_retry_non_string_initial_delay.rs".into(),
        "fail_retry_non_string_max_delay.rs".into(),
        "fail_retry_zero_exponential_base.rs".into(),
        "fail_retry_non_integer_exponential_base.rs".into(),
        "fail_retry_non_bool_jitter.rs".into(),
    ];

    ui_test::run_tests(config)
}
