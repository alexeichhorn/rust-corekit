use corekit::prelude::*;

struct Entry {
    name: String,
}

#[test]
fn consumes_non_clone_items_with_mutable_mapper_in_order() {
    let entries = vec![Entry { name: "Alex".into() }, Entry { name: "Sam".into() }];
    let mut calls = 0;

    let names: Vec<(usize, String)> = entries.into_map_vec(|entry| {
        calls += 1;
        (calls, entry.name)
    });

    assert_eq!(calls, 2);
    assert_eq!(names, vec![(1, "Alex".into()), (2, "Sam".into())]);
}

#[test]
fn output_can_borrow_from_a_slice() {
    let names = vec![String::from("Alex"), String::from("Sam")];

    let borrowed: Vec<&str> = names.as_slice().into_map_vec(|name| name.as_str());

    assert_eq!(borrowed, vec!["Alex", "Sam"]);
    assert!(std::ptr::eq(borrowed[0], names[0].as_str()));
    assert!(std::ptr::eq(borrowed[1], names[1].as_str()));
}

#[test]
fn consumes_owned_arrays() {
    let entries = [Entry { name: "Alex".into() }];

    let names: Vec<String> = entries.into_map_vec(|entry| entry.name);

    assert_eq!(names, vec![String::from("Alex")]);
}

#[test]
fn maps_iterator_chains() {
    let values = [1, 2, 3, 4];

    let mapped: Vec<i32> = values.into_iter().filter(|value| value % 2 == 0).into_map_vec(|value| value * 10);

    assert_eq!(mapped, vec![20, 40]);
}
