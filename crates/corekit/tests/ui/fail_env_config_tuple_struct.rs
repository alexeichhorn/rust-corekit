use corekit::EnvConfig;

#[derive(EnvConfig)]
struct TupleEnv(String);
//~^ ERROR: `EnvConfig` can only be derived for structs with named fields

fn main() {}
