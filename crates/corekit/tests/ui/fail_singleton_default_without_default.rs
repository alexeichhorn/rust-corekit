use corekit::singleton;

#[singleton(default)]
struct MissingDefault;
//~^ ERROR: no function or associated item named `default` found for struct `MissingDefault`

fn main() {
    let _service = MissingDefault::shared();
}
