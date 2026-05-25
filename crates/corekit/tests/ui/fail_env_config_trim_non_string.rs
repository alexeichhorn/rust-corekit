use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(trim)]
    //~^ ERROR: `#[env(trim)]` requires a `String` or `Option<String>` field
    worker_count: u16,
}

fn main() {}
