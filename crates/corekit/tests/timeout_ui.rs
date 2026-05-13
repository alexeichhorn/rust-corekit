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
        "timeout_basic.rs".into(),
        "timeout_from_prelude.rs".into(),
        "timeout_duration_units.rs".into(),
        "timeout_with_retry.rs".into(),
        "fail_timeout_non_async.rs".into(),
        "fail_timeout_missing_duration.rs".into(),
        "fail_timeout_extra_arg.rs".into(),
        "fail_timeout_non_string_duration.rs".into(),
        "fail_timeout_invalid_duration.rs".into(),
    ];

    ui_test::run_tests(config)
}
