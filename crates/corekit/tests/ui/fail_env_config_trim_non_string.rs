use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(trim)]
    //~^ ERROR: `#[env(trim)]` requires a `String`, `Option<String>`, `Vec<T>`, or `Option<Vec<T>>` field
    worker_count: u16,
}

fn main() {}
