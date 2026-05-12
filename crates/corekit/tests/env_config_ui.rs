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
        "env_config_basic.rs".into(),
        "env_config_from_prelude.rs".into(),
        "env_config_global_access.rs".into(),
        "env_config_global_visibility.rs".into(),
        "env_config_name_override.rs".into(),
        "env_config_uppercase_fields.rs".into(),
        "fail_env_config_enum.rs".into(),
        "fail_env_config_tuple_struct.rs".into(),
        "fail_env_config_unit_struct.rs".into(),
        "fail_env_config_global_string_name.rs".into(),
        "fail_env_config_unknown_container_arg.rs".into(),
        "fail_env_config_unknown_field_arg.rs".into(),
    ];

    ui_test::run_tests(config)
}
