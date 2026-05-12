//@check-pass
use corekit::singleton;
use std::sync::RwLock;

#[singleton]
pub struct UserService {
    cache: RwLock<Vec<String>>,
}

impl UserService {
    fn new() -> Self {
        Self {
            cache: RwLock::new(Vec::new()),
        }
    }

    pub fn count(&self) -> usize {
        self.cache.read().unwrap().len()
    }
}

fn main() {
    let service: &'static UserService = UserService::shared();
    let _count = service.count();
}
