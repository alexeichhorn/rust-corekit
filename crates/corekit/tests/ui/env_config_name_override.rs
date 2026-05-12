//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(name = "DATABASE_URL")]
    db_url: String,
}

fn main() {
    let _loaded = Env::load();
}
