use std::collections::HashMap;

use corekit::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    id: u64,
    name: String,
}

#[test]
fn indexes_owned_vec_without_clone_and_keeps_last_value_for_duplicate_keys() {
    let entries = vec![
        Entry { id: 1, name: "old".into() },
        Entry {
            id: 2,
            name: "other".into(),
        },
        Entry { id: 1, name: "new".into() },
    ];

    let indexed = entries.key_by(|entry| entry.id);

    assert_eq!(
        indexed,
        HashMap::from([
            (1, Entry { id: 1, name: "new".into() }),
            (
                2,
                Entry {
                    id: 2,
                    name: "other".into()
                }
            ),
        ])
    );
}

#[test]
fn slice_keys_and_values_can_borrow_from_the_input() {
    let entries = vec![
        Entry {
            id: 1,
            name: "Alex".into(),
        },
        Entry { id: 2, name: "Sam".into() },
    ];

    let indexed: HashMap<&str, &Entry> = entries.as_slice().key_by(|entry| entry.name.as_str());

    assert_eq!(indexed.len(), 2);
    assert!(std::ptr::eq(indexed["Alex"], &entries[0]));
    assert!(std::ptr::eq(indexed["Sam"], &entries[1]));
}

#[test]
fn indexes_owned_arrays() {
    let entries = [Entry {
        id: 1,
        name: "Alex".into(),
    }];

    let indexed: HashMap<u64, Entry> = entries.key_by(|entry| entry.id);

    assert_eq!(
        indexed,
        HashMap::from([(
            1,
            Entry {
                id: 1,
                name: "Alex".into()
            }
        )])
    );
}

#[test]
fn iterator_chain_calls_mutable_key_function_once_per_item_in_order() {
    let values = [10, 11, 20, 21, 30];
    let mut calls = 0;

    let indexed = values.iter().filter(|value| **value % 10 == 0).key_by(|_| {
        calls += 1;
        calls
    });

    assert_eq!(calls, 3);
    assert_eq!(indexed, HashMap::from([(1, &values[0]), (2, &values[2]), (3, &values[4])]));
}

#[test]
fn empty_input_returns_empty_map_without_calling_key_function() {
    let entries = Vec::<Entry>::new();

    let indexed: HashMap<u64, Entry> = entries.key_by(|_| panic!("empty input has no keys"));

    assert!(indexed.is_empty());
}
