use corekit::SecretString;

fn main() {
    let secret = SecretString::from("sk-test-secret");
    let _raw: String = secret.into();
    //~^ ERROR: the trait bound `String: From<SecretString>` is not satisfied
}
