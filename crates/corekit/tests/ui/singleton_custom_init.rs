//@check-pass
use corekit::singleton;

#[singleton(init = CustomInitService::build)]
pub struct CustomInitService {
    value: usize,
}

impl CustomInitService {
    fn build() -> Self {
        Self { value: 42 }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

fn main() {
    let service: &'static CustomInitService = CustomInitService::shared();
    let _value = service.value();
}
