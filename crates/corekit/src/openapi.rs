//! Poem OpenAPI helpers.

pub fn with_prefix(prefix: &str, path: &str) -> String {
    if path == "/" {
        prefix.to_owned()
    } else {
        format!("{prefix}{path}")
    }
}

#[macro_export]
macro_rules! route_group {
    ($prefix:literal, $api:expr) => {{
        struct FeatureApi<T>(T);

        impl<T> $crate::__private::poem_openapi::OpenApi for FeatureApi<T>
        where
            T: $crate::__private::poem_openapi::OpenApi,
        {
            fn meta() -> Vec<$crate::__private::poem_openapi::registry::MetaApi> {
                let mut meta = T::meta();

                for api in &mut meta {
                    for path in &mut api.paths {
                        path.path = $crate::openapi::with_prefix($prefix, path.path.as_str());
                    }
                }

                meta
            }

            fn register(registry: &mut $crate::__private::poem_openapi::registry::Registry) {
                T::register(registry);
            }

            fn add_routes(
                self,
                route_table: &mut ::std::collections::HashMap<
                    String,
                    ::std::collections::HashMap<
                        $crate::__private::poem::http::Method,
                        $crate::__private::poem::endpoint::BoxEndpoint<'static>,
                    >,
                >,
            ) {
                let mut inner_routes = ::std::collections::HashMap::new();
                self.0.add_routes(&mut inner_routes);

                for (path, methods) in inner_routes {
                    route_table
                        .entry($crate::openapi::with_prefix($prefix, path.as_str()))
                        .or_default()
                        .extend(methods);
                }
            }
        }

        FeatureApi($api)
    }};
}

pub use crate::route_group;
