use corekit::MapVec;

#[test]
fn maps_in_order_with_mutable_mapper_and_output_borrowing_from_input() {
    let names = vec![String::from("Alex"), String::from("Sam")];
    let mut calls = 0;

    let mapped: Result<Vec<(usize, &str)>, &str> = names.try_map_vec(|name| {
        calls += 1;
        Ok((calls, name.as_str()))
    });

    let mapped = mapped.unwrap();
    assert_eq!(calls, 2);
    assert_eq!(mapped, vec![(1, "Alex"), (2, "Sam")]);
    assert!(std::ptr::eq(mapped[0].1, names[0].as_str()));
    assert!(std::ptr::eq(mapped[1].1, names[1].as_str()));
}

#[test]
fn returns_first_error_without_mapping_later_items() {
    let values = [1, 2, 3];
    let mut visited = Vec::new();

    let mapped: Result<Vec<i32>, i32> = values.try_map_vec(|&value| {
        visited.push(value);
        if value >= 2 {
            Err(value)
        } else {
            Ok(value * 10)
        }
    });

    assert_eq!(mapped, Err(2));
    assert_eq!(visited, vec![1, 2]);
    assert_eq!(values, [1, 2, 3]);
}
