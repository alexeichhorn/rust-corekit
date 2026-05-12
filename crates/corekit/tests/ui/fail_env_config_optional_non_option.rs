use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(optional)]
    //~^ ERROR: `#[env(optional)]` requires an `Option<T>` field
    database_url: String,
}

fn main() {}
