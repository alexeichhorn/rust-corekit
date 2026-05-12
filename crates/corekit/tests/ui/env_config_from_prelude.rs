//@check-pass
use corekit::prelude::*;

#[derive(EnvConfig)]
#[env_config(global = env_from_prelude)]
struct EnvFromPrelude {
    database_url: String,
}

fn main() {
    let _loaded = EnvFromPrelude::load();
    let _database_url = env_from_prelude.database_url.as_str();
}
