use corekit::EnvConfig;

#[derive(EnvConfig)]
enum Env {
    //~^ ERROR: `EnvConfig` can only be derived for structs with named fields
    Variant,
}

fn main() {}
