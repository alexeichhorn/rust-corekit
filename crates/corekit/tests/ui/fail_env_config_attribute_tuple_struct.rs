use corekit::env_config;

#[env_config]
//~^ ERROR: `#[env_config]` can only be used on structs with named fields
struct Env(String);

fn main() {}
