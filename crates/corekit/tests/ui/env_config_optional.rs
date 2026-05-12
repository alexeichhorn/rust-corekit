//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    sentry_dsn: Option<String>,
    worker_count: Option<u16>,
    #[env(optional)]
    feature_enabled: Option<bool>,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
