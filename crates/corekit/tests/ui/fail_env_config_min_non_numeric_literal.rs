use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(min = "1")]
    //~^ ERROR: `#[env(min = ...)]` expects a numeric literal
    worker_count: u16,
}

fn main() {}
