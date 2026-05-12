//@check-pass
use corekit::singleton;

#[singleton(default)]
#[derive(Default)]
pub struct DefaultService {
    value: usize,
}

impl DefaultService {
    pub fn value(&self) -> usize {
        self.value
    }
}

fn main() {
    let service: &'static DefaultService = DefaultService::shared();
    let _value = service.value();
}
