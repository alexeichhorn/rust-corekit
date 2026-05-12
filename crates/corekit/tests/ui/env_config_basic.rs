//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    database_url: String,
    worker_count: u16,
    signed_limit: i32,
    feature_enabled: bool,
    ratio: f64,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
