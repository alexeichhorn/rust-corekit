use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(rename = "DATABASE_URL")]
    //~^ ERROR: unsupported `#[env]` argument
    database_url: String,
}

fn main() {}
