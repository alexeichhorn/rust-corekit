//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(trim)]
    database_url: String,
    #[env(non_empty)]
    api_key: String,
    #[env(trim, non_empty)]
    sentry_dsn: Option<String>,
    #[env(default = "  info  ", trim, non_empty)]
    log_level: String,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
