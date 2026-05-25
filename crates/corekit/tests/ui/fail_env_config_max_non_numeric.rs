use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(max = 1)]
    //~^ ERROR: `#[env(max = ...)]` requires an integer or float field
    feature_enabled: bool,
}

fn main() {}
