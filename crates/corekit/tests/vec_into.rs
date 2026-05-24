use corekit::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct UserDto {
    id: u64,
}

struct UserModel {
    id: u64,
}

impl From<UserModel> for UserDto {
    fn from(value: UserModel) -> Self {
        Self { id: value.id }
    }
}

#[test]
fn converts_vec_elements_with_from_impl() {
    let models = vec![UserModel { id: 1 }, UserModel { id: 2 }];

    let dtos: Vec<UserDto> = models.vec_into();

    assert_eq!(dtos, vec![UserDto { id: 1 }, UserDto { id: 2 }]);
}

#[test]
fn converts_empty_vec() {
    let models: Vec<UserModel> = Vec::new();

    let dtos: Vec<UserDto> = models.vec_into();

    assert!(dtos.is_empty());
}
