//@check-pass
use corekit::singleton;

#[singleton]
struct PrivateNewService;

impl PrivateNewService {
    fn new() -> Self {
        Self
    }
}

fn main() {
    let _service = PrivateNewService::shared();
}
