use corekit::EnvConfig;

#[derive(EnvConfig)]
struct FilterEmptyNonVec {
    #[env(filter_empty)]
    //~^ ERROR: `#[env(filter_empty)]` requires a `Vec<T>` or `Option<Vec<T>>` field
    database_url: String,
}

#[derive(EnvConfig)]
struct MinLengthNonVec {
    #[env(min_length = 1)]
    //~^ ERROR: `#[env(min_length = ...)]` requires a `Vec<T>` or `Option<Vec<T>>` field
    database_url: String,
}

#[derive(EnvConfig)]
struct MaxLengthNonVec {
    #[env(max_length = 1)]
    //~^ ERROR: `#[env(max_length = ...)]` requires a `Vec<T>` or `Option<Vec<T>>` field
    database_url: String,
}

#[derive(EnvConfig)]
struct MinOnVec {
    #[env(min = 1)]
    //~^ ERROR: `#[env(min = ...)]` requires an integer or float field
    ports: Vec<u16>,
}

#[derive(EnvConfig)]
struct EachMinOnScalar {
    #[env(each_min = 1)]
    //~^ ERROR: `#[env(each_min = ...)]` requires a `Vec<T>` or `Option<Vec<T>>` field with an integer or float item type
    worker_count: u16,
}

#[derive(EnvConfig)]
struct EachMaxOnStringVec {
    #[env(each_max = 1)]
    //~^ ERROR: `#[env(each_max = ...)]` requires a `Vec<T>` or `Option<Vec<T>>` field with an integer or float item type
    names: Vec<String>,
}

#[derive(EnvConfig)]
struct InvalidLengthLiteral {
    #[env(min_length = -1)]
    //~^ ERROR: `#[env(min_length = ...)]` expects a non-negative integer literal
    names: Vec<String>,
}

fn main() {}
