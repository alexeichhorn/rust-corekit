//@check-pass
use corekit::prelude::*;

#[env_config(dotenv = false)]
pub struct Env {
    database_url: String,
    #[env(default = 2)]
    worker_count: u16,
    sentry_dsn: Option<String>,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
