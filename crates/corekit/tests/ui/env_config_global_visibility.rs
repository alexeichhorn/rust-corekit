//@check-pass
mod config {
    use corekit::EnvConfig;

    #[derive(EnvConfig)]
    #[env_config(global = env)]
    #[allow(non_snake_case)]
    pub struct Env {
        pub DATABASE_URL: String,
    }
}

fn main() {
    let _database_url: &str = config::env.DATABASE_URL.as_str();
}
