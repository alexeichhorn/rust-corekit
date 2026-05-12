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
        "singleton_basic.rs".into(),
        "singleton_private_new.rs".into(),
        "singleton_from_prelude.rs".into(),
        "singleton_default.rs".into(),
        "singleton_custom_init.rs".into(),
        "fail_singleton_enum.rs".into(),
        "fail_singleton_missing_new.rs".into(),
        "fail_singleton_new_with_args.rs".into(),
        "fail_singleton_generic_struct.rs".into(),
        "fail_singleton_default_without_default.rs".into(),
        "fail_singleton_init_with_args.rs".into(),
        "fail_singleton_default_and_init.rs".into(),
    ];

    ui_test::run_tests(config)
}
