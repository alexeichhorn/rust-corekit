use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(min = 1)]
    //~^ ERROR: `#[env(min = ...)]` requires an integer or float field
    database_url: String,
}

fn main() {}
