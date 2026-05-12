//@check-pass
use corekit::EnvConfig;

#[derive(EnvConfig)]
struct Env {
    #[env(default = 90)]
    video_max_duration_s: u64,
    #[env(default = false)]
    dev_mode: bool,
    #[env(default = "info")]
    log_level: String,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
