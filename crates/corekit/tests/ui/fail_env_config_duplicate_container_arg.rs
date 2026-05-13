use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(global = first_env, global = second_env)]
//~^ ERROR: duplicate `#[env_config(global = ...)]` argument
struct DuplicateGlobalEnv {
    database_url: String,
}

#[derive(EnvConfig)]
#[env_config(dotenv = ".env.test", dotenv = false)]
//~^ ERROR: duplicate `#[env_config(dotenv = ...)]` argument
struct DuplicateDotenvEnv {
    database_url: String,
}

fn main() {}
