use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(cache = env)]
//~^ ERROR: unsupported `#[env_config]` argument
struct Env {
    database_url: String,
}

fn main() {}
