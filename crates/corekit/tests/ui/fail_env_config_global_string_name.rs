use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(global = "env")]
//~^ ERROR: `#[env_config(global = ...)]` expects an identifier
struct Env {
    database_url: String,
}

fn main() {}
