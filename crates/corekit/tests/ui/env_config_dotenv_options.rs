//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
#[env_config(dotenv = ".env.test")]
struct CustomDotenvEnv {
    database_url: String,
}

#[derive(EnvConfig)]
#[env_config(dotenv = false)]
struct DisabledDotenvEnv {
    database_url: String,
}

fn main() {
    let _custom: Result<CustomDotenvEnv, corekit::EnvError> = CustomDotenvEnv::load();
    let _disabled: Result<DisabledDotenvEnv, corekit::EnvError> = DisabledDotenvEnv::load();
}
