use corekit::EnvConfig;

const DEFAULT_WORKER_COUNT: u16 = 4;

#[derive(EnvConfig)]
struct Env {
    #[env(default = DEFAULT_WORKER_COUNT)]
    //~^ ERROR: `#[env(default = ...)]` expects a literal
    worker_count: u16,
}

fn main() {}
