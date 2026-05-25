use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(non_empty)]
    //~^ ERROR: `#[env(non_empty)]` requires a `String` or `Option<String>` field
    worker_count: Option<u16>,
}

fn main() {}
