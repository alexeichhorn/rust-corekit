use corekit::singleton;

#[singleton]
struct MissingNew;
//~^ ERROR: no function or associated item named `new` found for struct `MissingNew`

fn main() {
    let _service = MissingNew::shared();
}
