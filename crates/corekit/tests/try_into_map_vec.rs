use corekit::IntoMapVec;

struct Entry {
    name: String,
}

#[test]
fn consumes_non_clone_items_with_mutable_mapper_in_order() {
    let entries = vec![Entry { name: "Alex".into() }, Entry { name: "Sam".into() }];
    let mut calls = 0;

    let mapped: Result<Vec<(usize, String)>, &str> = entries.try_into_map_vec(|entry| {
        calls += 1;
        Ok((calls, entry.name))
    });

    assert_eq!(calls, 2);
    assert_eq!(mapped, Ok(vec![(1, "Alex".into()), (2, "Sam".into())]));
}

#[test]
fn iterator_chain_returns_first_error_without_mapping_later_items() {
    let mut visited = Vec::new();

    let mapped: Result<Vec<i32>, i32> = (1..=6).filter(|value| value % 2 == 0).try_into_map_vec(|value| {
        visited.push(value);
        if value >= 4 {
            Err(value)
        } else {
            Ok(value * 10)
        }
    });

    assert_eq!(mapped, Err(4));
    assert_eq!(visited, vec![2, 4]);
}
