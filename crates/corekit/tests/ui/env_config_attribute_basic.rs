//@check-pass
use corekit::env_config;

#[env_config]
pub struct Env {
    database_url: String,
    worker_count: u16,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
