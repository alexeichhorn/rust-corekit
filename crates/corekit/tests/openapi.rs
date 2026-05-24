#![cfg(feature = "poem-openapi")]

use std::collections::HashMap;

use corekit::{openapi::with_prefix, route_group};
use poem::endpoint::BoxEndpoint;
use poem::http::Method;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};

#[derive(Object)]
struct StatusResponse {
    ok: bool,
}

struct RootRoutes;

#[OpenApi]
impl RootRoutes {
    #[oai(path = "/", method = "get")]
    async fn status(&self) -> Json<StatusResponse> {
        Json(StatusResponse { ok: true })
    }
}

struct ActivityRoutes;

#[OpenApi(prefix_path = "/activity")]
impl ActivityRoutes {
    #[oai(path = "/:todo_id", method = "get")]
    async fn list_for_todo(&self) -> Json<StatusResponse> {
        Json(StatusResponse { ok: true })
    }
}

struct TodoCollectionRoutes;

#[OpenApi]
impl TodoCollectionRoutes {
    #[oai(path = "/todos", method = "post")]
    async fn create_todo(&self) -> Json<StatusResponse> {
        Json(StatusResponse { ok: true })
    }
}

fn meta_paths<T: OpenApi>(_: &T) -> Vec<String> {
    let mut paths = T::meta()
        .into_iter()
        .flat_map(|api| api.paths)
        .map(|path| path.path)
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn route_paths<T: OpenApi>(api: T) -> Vec<String> {
    let mut route_table: HashMap<String, HashMap<Method, BoxEndpoint<'static>>> = HashMap::new();
    api.add_routes(&mut route_table);

    let mut paths = route_table.into_keys().collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn with_prefix_replaces_root_path_with_prefix() {
    assert_eq!(with_prefix("/todos", "/"), "/todos");
    assert_eq!(with_prefix("/todos", "/activity"), "/todos/activity");
}

#[test]
fn route_group_prefixes_single_api_meta_and_routes() {
    let api = route_group!("/health", RootRoutes);

    assert_eq!(meta_paths(&api), vec!["/health"]);
    assert_eq!(route_paths(api), vec!["/health"]);
}

#[test]
fn route_group_can_be_imported_from_openapi_module() {
    use corekit::openapi::route_group as prefixed_routes;

    let api = prefixed_routes!("/health", RootRoutes);

    assert_eq!(meta_paths(&api), vec!["/health"]);
}

#[test]
fn route_group_prefixes_tuple_api_meta_and_routes() {
    let api = route_group!("/todos", (RootRoutes, ActivityRoutes));

    assert_eq!(meta_paths(&api), vec!["/todos", "/todos/activity/{todo_id}"]);
    assert_eq!(route_paths(api), vec!["/todos", "/todos/activity/:param0"]);
}

#[test]
fn route_group_preserves_existing_methods_for_prefixed_path_collisions() {
    let api = (TodoCollectionRoutes, route_group!("/todos", RootRoutes));

    let mut route_table: HashMap<String, HashMap<Method, BoxEndpoint<'static>>> = HashMap::new();
    api.add_routes(&mut route_table);

    let methods = route_table.get("/todos").expect("route exists");
    assert!(methods.contains_key(&Method::POST));
    assert!(methods.contains_key(&Method::GET));
}
