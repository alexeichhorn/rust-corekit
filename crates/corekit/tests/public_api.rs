#[test]
fn prelude_can_be_glob_imported() {
    mod consumer {
        #![allow(unused_imports)]

        use corekit::prelude::*;

        pub fn compiled() {}
    }

    consumer::compiled();
}
