//@check-pass
#![deny(warnings)]

use corekit::prelude::*;

#[env_config(global = env, dotenv = false)]
pub struct Env {
    pub DATABASE_URL: String,
    pub OPENAI_API_KEY: SecretString,
    #[env(default = false)]
    pub DEV_MODE: bool,
}

fn main() {
    let _database_url: &str = env.DATABASE_URL.as_str();
    let _openai_api_key: &str = env.OPENAI_API_KEY.expose();
    let _dev_mode: bool = env.DEV_MODE;
}
