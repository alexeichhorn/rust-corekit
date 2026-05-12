use corekit::EnvConfig;

#[derive(EnvConfig)]
struct UnitEnv;
//~^ ERROR: `EnvConfig` can only be derived for structs with named fields

fn main() {}
