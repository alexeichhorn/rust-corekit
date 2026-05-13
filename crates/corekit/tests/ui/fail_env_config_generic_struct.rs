use corekit::{env_config, EnvConfig};

#[derive(EnvConfig)]
struct DeriveEnv<T> {
    //~^ ERROR: `EnvConfig` does not support generic structs
    value: T,
}

#[env_config]
//~^ ERROR: `#[env_config]` does not support generic structs
struct AttributeEnv<T> {
    value: T,
}

fn main() {}
