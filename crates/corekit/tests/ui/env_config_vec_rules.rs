//@check-pass
use std::str::FromStr;

use corekit::{EnvConfig, SecretString};

#[derive(Debug)]
enum Mode {
    Fast,
    Safe,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fast" => Ok(Self::Fast),
            "safe" => Ok(Self::Safe),
            _ => Err(()),
        }
    }
}

#[derive(EnvConfig)]
struct Env {
    #[env(trim, filter_empty, non_empty, min_length = 1, max_length = 3)]
    names: Vec<String>,
    #[env(trim, filter_empty, each_min = 1, each_max = 10)]
    ports: Vec<u16>,
    #[env(trim, filter_empty)]
    flags: Option<Vec<bool>>,
    #[env(trim, filter_empty)]
    secrets: Vec<SecretString>,
    #[env(default = "fast,safe", trim, filter_empty)]
    modes: Vec<Mode>,
}

fn main() {
    let _loaded: Result<Env, corekit::EnvError> = Env::load();
}
