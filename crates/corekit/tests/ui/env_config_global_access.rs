//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(global = env)]
#[allow(non_snake_case)]
struct Env {
    DATABASE_URL: String,
    OPENAI_API_KEY: corekit::SecretString,
    DEV_MODE: bool,
    WORKER_COUNT: u16,
}

fn main() {
    let _database_url: &str = env.DATABASE_URL.as_str();
    let _openai_api_key: &str = env.OPENAI_API_KEY.expose();
    let _dev_mode: bool = env.DEV_MODE;
    let _worker_count: u16 = env.WORKER_COUNT;
}
