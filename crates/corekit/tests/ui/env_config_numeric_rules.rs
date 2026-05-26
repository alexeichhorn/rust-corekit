//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(min = 1, max = 10)]
    worker_count: u16,
    #[env(default = 4, min = 1, max = 8)]
    thread_limit: usize,
    #[env(min = -2, max = 2)]
    signed_limit: i32,
    #[env(min = 0.5, max = 1.5)]
    ratio: f64,
    #[env(min = 1024)]
    port: Option<u32>,
    #[env(default = 30, min = 1, max = 60)]
    timeout_seconds: u64,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
