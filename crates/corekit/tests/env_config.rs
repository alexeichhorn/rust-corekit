use std::env;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use corekit::prelude::*;

static ENV_LOCK: Mutex<()> = Mutex::new(());
static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, EnvConfig)]
struct RequiredPrimitives {
    corekit_test_service_url: String,
    corekit_test_worker_count: u16,
    corekit_test_signed_limit: i32,
    corekit_test_feature_enabled: bool,
    corekit_test_ratio: f64,
}

#[derive(Debug, EnvConfig)]
#[allow(non_snake_case)]
struct UppercaseRequiredPrimitives {
    COREKIT_UPPER_DATABASE_URL: String,
    COREKIT_UPPER_WORKER_COUNT: u16,
}

#[derive(Debug, EnvConfig)]
struct CustomNameEnv {
    #[env(name = "COREKIT_CUSTOM_DATABASE_URL")]
    database_url: String,
}

#[derive(Debug, EnvConfig)]
#[env_config(global = corekit_global_env)]
#[allow(non_snake_case)]
struct GlobalEnv {
    COREKIT_GLOBAL_DATABASE_URL: String,
    COREKIT_GLOBAL_OPENAI_API_KEY: SecretString,
    COREKIT_GLOBAL_DEV_MODE: bool,
    COREKIT_GLOBAL_WORKER_COUNT: u16,
}

#[derive(Debug, EnvConfig)]
#[env_config(global = corekit_global_once_env)]
#[allow(non_snake_case)]
struct GlobalOnceEnv {
    COREKIT_GLOBAL_ONCE_DATABASE_URL: String,
}

#[derive(Debug, EnvConfig)]
#[env_config(global = corekit_global_invalid_env)]
#[allow(non_snake_case)]
struct GlobalInvalidEnv {
    COREKIT_GLOBAL_INVALID_PORT: u16,
}

#[derive(Debug, EnvConfig)]
struct DefaultEnv {
    #[env(default = 90)]
    corekit_default_video_max_duration_s: u64,
    #[env(default = false)]
    corekit_default_dev_mode: bool,
    #[env(default = "info")]
    corekit_default_log_level: String,
}

#[derive(Debug, EnvConfig)]
struct OptionalEnv {
    corekit_optional_sentry_dsn: Option<String>,
    corekit_optional_worker_count: Option<u16>,
    #[env(optional)]
    corekit_optional_feature_enabled: Option<bool>,
}

#[derive(Debug, EnvConfig)]
struct DotenvEnv {
    corekit_dotenv_database_url: String,
    corekit_dotenv_worker_count: u16,
}

#[derive(Debug, EnvConfig)]
#[env_config(dotenv = ".env.test")]
struct CustomDotenvEnv {
    corekit_custom_dotenv_database_url: String,
}

#[derive(Debug, EnvConfig)]
#[env_config(dotenv = false)]
struct DisabledDotenvEnv {
    corekit_disabled_dotenv_database_url: String,
}

#[env_config(global = corekit_attribute_global_env, dotenv = false)]
pub struct AttributeEnv {
    pub COREKIT_ATTRIBUTE_DATABASE_URL: String,
    pub COREKIT_ATTRIBUTE_API_KEY: SecretString,
    #[env(default = false)]
    pub COREKIT_ATTRIBUTE_DEV_MODE: bool,
    pub COREKIT_ATTRIBUTE_SENTRY_DSN: Option<String>,
}

#[test]
fn load_reads_required_primitive_fields_from_screaming_snake_case_env_names() {
    with_env(
        &[
            ("COREKIT_TEST_SERVICE_URL", Some("https://example.com")),
            ("COREKIT_TEST_WORKER_COUNT", Some("12")),
            ("COREKIT_TEST_SIGNED_LIMIT", Some("-7")),
            ("COREKIT_TEST_FEATURE_ENABLED", Some("true")),
            ("COREKIT_TEST_RATIO", Some("0.75")),
        ],
        || {
            let config = RequiredPrimitives::load().unwrap();

            assert_eq!(config.corekit_test_service_url, "https://example.com");
            assert_eq!(config.corekit_test_worker_count, 12);
            assert_eq!(config.corekit_test_signed_limit, -7);
            assert!(config.corekit_test_feature_enabled);
            assert_eq!(config.corekit_test_ratio, 0.75);
        },
    );
}

#[test]
fn uppercase_fields_map_to_the_same_uppercase_env_names() {
    with_env(
        &[
            ("COREKIT_UPPER_DATABASE_URL", Some("postgres://upper")),
            ("COREKIT_UPPER_WORKER_COUNT", Some("3")),
        ],
        || {
            let config = UppercaseRequiredPrimitives::load().unwrap();

            assert_eq!(config.COREKIT_UPPER_DATABASE_URL, "postgres://upper");
            assert_eq!(config.COREKIT_UPPER_WORKER_COUNT, 3);
        },
    );
}

#[test]
fn env_name_attribute_overrides_the_default_field_mapping() {
    with_env(&[("COREKIT_CUSTOM_DATABASE_URL", Some("postgres://custom"))], || {
        let config = CustomNameEnv::load().unwrap();

        assert_eq!(config.database_url, "postgres://custom");
    });
}

#[test]
fn treats_empty_string_as_present_for_required_string_fields() {
    with_env(
        &[
            ("COREKIT_TEST_SERVICE_URL", Some("")),
            ("COREKIT_TEST_WORKER_COUNT", Some("1")),
            ("COREKIT_TEST_SIGNED_LIMIT", Some("0")),
            ("COREKIT_TEST_FEATURE_ENABLED", Some("false")),
            ("COREKIT_TEST_RATIO", Some("1.0")),
        ],
        || {
            let config = RequiredPrimitives::load().unwrap();

            assert_eq!(config.corekit_test_service_url, "");
            assert!(!config.corekit_test_feature_enabled);
        },
    );
}

#[test]
fn collects_all_missing_required_env_vars_in_field_order() {
    with_env(&all_env_vars_absent(), || {
        let error = RequiredPrimitives::load().unwrap_err();
        let errors = error.errors();

        assert_eq!(errors.len(), 5);
        assert_eq!(errors[0].name(), "COREKIT_TEST_SERVICE_URL");
        assert_eq!(errors[0].kind(), EnvErrorKind::Missing);
        assert_eq!(errors[1].name(), "COREKIT_TEST_WORKER_COUNT");
        assert_eq!(errors[1].kind(), EnvErrorKind::Missing);
        assert_eq!(errors[2].name(), "COREKIT_TEST_SIGNED_LIMIT");
        assert_eq!(errors[2].kind(), EnvErrorKind::Missing);
        assert_eq!(errors[3].name(), "COREKIT_TEST_FEATURE_ENABLED");
        assert_eq!(errors[3].kind(), EnvErrorKind::Missing);
        assert_eq!(errors[4].name(), "COREKIT_TEST_RATIO");
        assert_eq!(errors[4].kind(), EnvErrorKind::Missing);
    });
}

#[test]
fn collects_all_invalid_required_values_in_field_order() {
    with_env(
        &[
            ("COREKIT_TEST_SERVICE_URL", Some("https://example.com")),
            ("COREKIT_TEST_WORKER_COUNT", Some("-1")),
            ("COREKIT_TEST_SIGNED_LIMIT", Some("not-an-int")),
            ("COREKIT_TEST_FEATURE_ENABLED", Some("yes")),
            ("COREKIT_TEST_RATIO", Some("not-a-float")),
        ],
        || {
            let error = RequiredPrimitives::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 4);
            assert_eq!(errors[0].name(), "COREKIT_TEST_WORKER_COUNT");
            assert_eq!(errors[0].kind(), EnvErrorKind::Invalid);
            assert_eq!(errors[1].name(), "COREKIT_TEST_SIGNED_LIMIT");
            assert_eq!(errors[1].kind(), EnvErrorKind::Invalid);
            assert_eq!(errors[2].name(), "COREKIT_TEST_FEATURE_ENABLED");
            assert_eq!(errors[2].kind(), EnvErrorKind::Invalid);
            assert_eq!(errors[3].name(), "COREKIT_TEST_RATIO");
            assert_eq!(errors[3].kind(), EnvErrorKind::Invalid);
        },
    );
}

#[test]
fn reports_missing_and_invalid_values_together() {
    with_env(
        &[
            ("COREKIT_TEST_SERVICE_URL", None),
            ("COREKIT_TEST_WORKER_COUNT", Some("not-a-u16")),
            ("COREKIT_TEST_SIGNED_LIMIT", Some("10")),
            ("COREKIT_TEST_FEATURE_ENABLED", None),
            ("COREKIT_TEST_RATIO", Some("2.5")),
        ],
        || {
            let error = RequiredPrimitives::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 3);
            assert_eq!(errors[0].name(), "COREKIT_TEST_SERVICE_URL");
            assert_eq!(errors[0].kind(), EnvErrorKind::Missing);
            assert_eq!(errors[1].name(), "COREKIT_TEST_WORKER_COUNT");
            assert_eq!(errors[1].kind(), EnvErrorKind::Invalid);
            assert_eq!(errors[2].name(), "COREKIT_TEST_FEATURE_ENABLED");
            assert_eq!(errors[2].kind(), EnvErrorKind::Missing);
        },
    );
}

#[test]
fn error_display_lists_all_failed_env_var_names() {
    with_env(&all_env_vars_absent(), || {
        let message = RequiredPrimitives::load().unwrap_err().to_string();

        assert!(message.contains("COREKIT_TEST_SERVICE_URL"));
        assert!(message.contains("COREKIT_TEST_WORKER_COUNT"));
        assert!(message.contains("COREKIT_TEST_SIGNED_LIMIT"));
        assert!(message.contains("COREKIT_TEST_FEATURE_ENABLED"));
        assert!(message.contains("COREKIT_TEST_RATIO"));
    });
}

#[test]
fn global_env_can_be_read_with_field_access() {
    with_env(
        &[
            ("COREKIT_GLOBAL_DATABASE_URL", Some("postgres://global")),
            ("COREKIT_GLOBAL_OPENAI_API_KEY", Some("sk-global-secret")),
            ("COREKIT_GLOBAL_DEV_MODE", Some("true")),
            ("COREKIT_GLOBAL_WORKER_COUNT", Some("8")),
        ],
        || {
            let database_url: &str = corekit_global_env.COREKIT_GLOBAL_DATABASE_URL.as_str();
            let openai_api_key: &str = corekit_global_env.COREKIT_GLOBAL_OPENAI_API_KEY.expose();
            let dev_mode: bool = corekit_global_env.COREKIT_GLOBAL_DEV_MODE;
            let worker_count: u16 = corekit_global_env.COREKIT_GLOBAL_WORKER_COUNT;

            assert_eq!(database_url, "postgres://global");
            assert_eq!(openai_api_key, "sk-global-secret");
            assert!(dev_mode);
            assert_eq!(worker_count, 8);
        },
    );
}

#[test]
fn global_env_is_loaded_only_once_on_first_access() {
    with_env(&[("COREKIT_GLOBAL_ONCE_DATABASE_URL", Some("postgres://first"))], || {
        assert_eq!(corekit_global_once_env.COREKIT_GLOBAL_ONCE_DATABASE_URL, "postgres://first");

        env::set_var("COREKIT_GLOBAL_ONCE_DATABASE_URL", "postgres://second");

        assert_eq!(corekit_global_once_env.COREKIT_GLOBAL_ONCE_DATABASE_URL, "postgres://first");
    });
}

#[test]
fn global_env_panics_with_collected_errors_when_lazy_load_fails() {
    with_env(&[("COREKIT_GLOBAL_INVALID_PORT", Some("not-a-port"))], || {
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = corekit_global_invalid_env.COREKIT_GLOBAL_INVALID_PORT;
        }))
        .unwrap_err();
        let message = panic_message(panic.as_ref());

        assert!(message.contains("failed to load EnvConfig"));
        assert!(message.contains("COREKIT_GLOBAL_INVALID_PORT"));
    });
}

#[test]
fn defaults_are_used_when_env_vars_are_missing() {
    with_env(&all_default_env_vars_absent(), || {
        let config = DefaultEnv::load().unwrap();

        assert_eq!(config.corekit_default_video_max_duration_s, 90);
        assert!(!config.corekit_default_dev_mode);
        assert_eq!(config.corekit_default_log_level, "info");
    });
}

#[test]
fn env_values_override_defaults_when_present() {
    with_env(
        &[
            ("COREKIT_DEFAULT_VIDEO_MAX_DURATION_S", Some("120")),
            ("COREKIT_DEFAULT_DEV_MODE", Some("true")),
            ("COREKIT_DEFAULT_LOG_LEVEL", Some("debug")),
        ],
        || {
            let config = DefaultEnv::load().unwrap();

            assert_eq!(config.corekit_default_video_max_duration_s, 120);
            assert!(config.corekit_default_dev_mode);
            assert_eq!(config.corekit_default_log_level, "debug");
        },
    );
}

#[test]
fn present_invalid_values_fail_even_when_defaults_exist() {
    with_env(
        &[
            ("COREKIT_DEFAULT_VIDEO_MAX_DURATION_S", Some("not-a-duration")),
            ("COREKIT_DEFAULT_DEV_MODE", Some("true")),
            ("COREKIT_DEFAULT_LOG_LEVEL", None),
        ],
        || {
            let error = DefaultEnv::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].name(), "COREKIT_DEFAULT_VIDEO_MAX_DURATION_S");
            assert_eq!(errors[0].kind(), EnvErrorKind::Invalid);
        },
    );
}

#[test]
fn missing_option_fields_load_as_none() {
    with_env(&all_optional_env_vars_absent(), || {
        let config = OptionalEnv::load().unwrap();

        assert_eq!(config.corekit_optional_sentry_dsn, None);
        assert_eq!(config.corekit_optional_worker_count, None);
        assert_eq!(config.corekit_optional_feature_enabled, None);
    });
}

#[test]
fn present_option_fields_load_as_some_values() {
    with_env(
        &[
            ("COREKIT_OPTIONAL_SENTRY_DSN", Some("https://sentry.example")),
            ("COREKIT_OPTIONAL_WORKER_COUNT", Some("7")),
            ("COREKIT_OPTIONAL_FEATURE_ENABLED", Some("true")),
        ],
        || {
            let config = OptionalEnv::load().unwrap();

            assert_eq!(config.corekit_optional_sentry_dsn.as_deref(), Some("https://sentry.example"));
            assert_eq!(config.corekit_optional_worker_count, Some(7));
            assert_eq!(config.corekit_optional_feature_enabled, Some(true));
        },
    );
}

#[test]
fn present_invalid_option_values_fail() {
    with_env(
        &[
            ("COREKIT_OPTIONAL_SENTRY_DSN", None),
            ("COREKIT_OPTIONAL_WORKER_COUNT", Some("not-a-u16")),
            ("COREKIT_OPTIONAL_FEATURE_ENABLED", Some("yes")),
        ],
        || {
            let error = OptionalEnv::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 2);
            assert_eq!(errors[0].name(), "COREKIT_OPTIONAL_WORKER_COUNT");
            assert_eq!(errors[0].kind(), EnvErrorKind::Invalid);
            assert_eq!(errors[1].name(), "COREKIT_OPTIONAL_FEATURE_ENABLED");
            assert_eq!(errors[1].kind(), EnvErrorKind::Invalid);
        },
    );
}

#[test]
fn load_reads_values_from_dotenv_file() {
    with_env(&all_dotenv_env_vars_absent(), || {
        with_dotenv(
            "COREKIT_DOTENV_DATABASE_URL=postgres://dotenv\nCOREKIT_DOTENV_WORKER_COUNT=4\n",
            || {
                let config = DotenvEnv::load().unwrap();

                assert_eq!(config.corekit_dotenv_database_url, "postgres://dotenv");
                assert_eq!(config.corekit_dotenv_worker_count, 4);
            },
        );
    });
}

#[test]
fn process_env_values_override_dotenv_values() {
    with_env(
        &[
            ("COREKIT_DOTENV_DATABASE_URL", Some("postgres://process")),
            ("COREKIT_DOTENV_WORKER_COUNT", None),
        ],
        || {
            with_dotenv(
                "COREKIT_DOTENV_DATABASE_URL=postgres://dotenv\nCOREKIT_DOTENV_WORKER_COUNT=4\n",
                || {
                    let config = DotenvEnv::load().unwrap();

                    assert_eq!(config.corekit_dotenv_database_url, "postgres://process");
                    assert_eq!(config.corekit_dotenv_worker_count, 4);
                },
            );
        },
    );
}

#[test]
fn custom_dotenv_filename_is_loaded_when_configured() {
    with_env(&[("COREKIT_CUSTOM_DOTENV_DATABASE_URL", None)], || {
        with_dotenv_file(".env.test", "COREKIT_CUSTOM_DOTENV_DATABASE_URL=postgres://custom-dotenv\n", || {
            let config = CustomDotenvEnv::load().unwrap();

            assert_eq!(config.corekit_custom_dotenv_database_url, "postgres://custom-dotenv");
        });
    });
}

#[test]
fn default_dotenv_file_is_not_loaded_when_custom_dotenv_filename_is_configured() {
    with_env(&[("COREKIT_CUSTOM_DOTENV_DATABASE_URL", None)], || {
        with_dotenv("COREKIT_CUSTOM_DOTENV_DATABASE_URL=postgres://default-dotenv\n", || {
            let error = CustomDotenvEnv::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].name(), "COREKIT_CUSTOM_DOTENV_DATABASE_URL");
            assert_eq!(errors[0].kind(), EnvErrorKind::Missing);
        });
    });
}

#[test]
fn dotenv_loading_can_be_disabled() {
    with_env(&[("COREKIT_DISABLED_DOTENV_DATABASE_URL", None)], || {
        with_dotenv("COREKIT_DISABLED_DOTENV_DATABASE_URL=postgres://disabled-dotenv\n", || {
            let error = DisabledDotenvEnv::load().unwrap_err();
            let errors = error.errors();

            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].name(), "COREKIT_DISABLED_DOTENV_DATABASE_URL");
            assert_eq!(errors[0].kind(), EnvErrorKind::Missing);
        });
    });
}

#[test]
fn process_env_values_still_work_when_dotenv_loading_is_disabled() {
    with_env(&[("COREKIT_DISABLED_DOTENV_DATABASE_URL", Some("postgres://process"))], || {
        with_dotenv("COREKIT_DISABLED_DOTENV_DATABASE_URL=postgres://disabled-dotenv\n", || {
            let config = DisabledDotenvEnv::load().unwrap();

            assert_eq!(config.corekit_disabled_dotenv_database_url, "postgres://process");
        });
    });
}

#[test]
fn attribute_macro_loads_env_and_generates_global_access() {
    with_env(
        &[
            ("COREKIT_ATTRIBUTE_DATABASE_URL", Some("postgres://attribute")),
            ("COREKIT_ATTRIBUTE_API_KEY", Some("sk-attribute")),
            ("COREKIT_ATTRIBUTE_DEV_MODE", None),
            ("COREKIT_ATTRIBUTE_SENTRY_DSN", None),
        ],
        || {
            let loaded = AttributeEnv::load().unwrap();

            assert_eq!(loaded.COREKIT_ATTRIBUTE_DATABASE_URL, "postgres://attribute");
            assert_eq!(loaded.COREKIT_ATTRIBUTE_API_KEY.expose(), "sk-attribute");
            assert!(!loaded.COREKIT_ATTRIBUTE_DEV_MODE);
            assert_eq!(loaded.COREKIT_ATTRIBUTE_SENTRY_DSN, None);

            let database_url: &str = corekit_attribute_global_env.COREKIT_ATTRIBUTE_DATABASE_URL.as_str();
            let api_key: &str = corekit_attribute_global_env.COREKIT_ATTRIBUTE_API_KEY.expose();

            assert_eq!(database_url, "postgres://attribute");
            assert_eq!(api_key, "sk-attribute");
        },
    );
}

fn all_env_vars_absent() -> [(&'static str, Option<&'static str>); 5] {
    [
        ("COREKIT_TEST_SERVICE_URL", None),
        ("COREKIT_TEST_WORKER_COUNT", None),
        ("COREKIT_TEST_SIGNED_LIMIT", None),
        ("COREKIT_TEST_FEATURE_ENABLED", None),
        ("COREKIT_TEST_RATIO", None),
    ]
}

fn all_default_env_vars_absent() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("COREKIT_DEFAULT_VIDEO_MAX_DURATION_S", None),
        ("COREKIT_DEFAULT_DEV_MODE", None),
        ("COREKIT_DEFAULT_LOG_LEVEL", None),
    ]
}

fn all_optional_env_vars_absent() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("COREKIT_OPTIONAL_SENTRY_DSN", None),
        ("COREKIT_OPTIONAL_WORKER_COUNT", None),
        ("COREKIT_OPTIONAL_FEATURE_ENABLED", None),
    ]
}

fn all_dotenv_env_vars_absent() -> [(&'static str, Option<&'static str>); 2] {
    [("COREKIT_DOTENV_DATABASE_URL", None), ("COREKIT_DOTENV_WORKER_COUNT", None)]
}

fn with_env(vars: &[(&'static str, Option<&'static str>)], test: impl FnOnce()) {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::new(vars);

    test();
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }

    if let Some(message) = panic.downcast_ref::<&'static str>() {
        return (*message).to_owned();
    }

    String::new()
}

fn with_dotenv(contents: &str, test: impl FnOnce()) {
    with_dotenv_file(".env", contents, test);
}

fn with_dotenv_file(filename: &str, contents: &str, test: impl FnOnce()) {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(filename), contents).unwrap();

    let _guard = CurrentDirGuard::new(dir);

    test();
}

fn unique_temp_dir() -> PathBuf {
    let index = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!("corekit-env-config-{}-{index}", process::id()))
}

struct CurrentDirGuard {
    original: PathBuf,
    temp: PathBuf,
}

impl CurrentDirGuard {
    fn new(temp: PathBuf) -> Self {
        let original = env::current_dir().unwrap();
        env::set_current_dir(&temp).unwrap();

        Self { original, temp }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).unwrap();
        fs::remove_dir_all(&self.temp).unwrap();
    }
}

struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new(vars: &[(&'static str, Option<&'static str>)]) -> Self {
        let saved = vars.iter().map(|(name, _)| (*name, env::var(name).ok())).collect();

        for (name, value) in vars {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }

        Self { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (name, value) in self.saved.drain(..) {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }
}
