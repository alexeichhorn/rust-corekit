use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(dotenv = true)]
//~^ ERROR: `#[env_config(dotenv = ...)]` expects a string literal or `false`
struct BoolTrueEnv {
    database_url: String,
}

#[derive(EnvConfig)]
#[env_config(dotenv = 123)]
//~^ ERROR: `#[env_config(dotenv = ...)]` expects a string literal or `false`
struct NumberEnv {
    database_url: String,
}

fn main() {}
