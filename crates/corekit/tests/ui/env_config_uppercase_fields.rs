//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
#[allow(non_snake_case)]
struct Env {
    DATABASE_URL: String,
    OPENAI_API_KEY: corekit::SecretString,
}

fn main() {
    let _loaded = Env::load();
}
