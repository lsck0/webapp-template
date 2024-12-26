//! Request macros, they are not necessarily better, but are a different way to write the same
//! code.

use serde::Serialize;

#[allow(unused)]
pub enum Payload<T: Serialize> {
    Body(T),
    QueryParams(T),
}

/// Make a request.
macro_rules! make_request {
    ($server:expr, $method:expr, $path:expr) => {
        $server.method($method, $path).await
    };

    ($server:expr, $method:expr, $path:expr, $payload:expr) => {{
        use shared::Payload::*;
        match $payload {
            Body(body) => $server.method($method, $path).json(&body).await,
            QueryParams(query_params) => $server.method($method, $path).add_query_params(&query_params).await,
        }
    }};
}
#[allow(unused)]
pub(crate) use make_request;

/// Check the response status code.
macro_rules! check_request {
    ($server:expr, $method:expr, $path:expr, $status_code:expr) => {
        let response = shared::make_request!($server, $method, $path);
        pretty_assertions::assert_eq!(response.status_code(), $status_code);
    };

    ($server:expr, $method:expr, $path:expr, $payload:expr, $status_code:expr) => {
        let response = shared::make_request!($server, $method, $path, $payload);
        pretty_assertions::assert_eq!(response.status_code(), $status_code);
    };
}
#[allow(unused)]
pub(crate) use check_request;

/// Make a request and check the response status code return the body.
macro_rules! get_response {
    ($server:expr, $method:expr, $path:expr, $status_code:expr, $return_t:ty) => {{
        let response = shared::make_request!($server, $method, $path);
        assert_eq!(response.status_code(), $status_code);

        response.json::<$return_t>()
    }};

    ($server:expr, $method:expr, $path:expr, $payload:expr, $status_code:expr, $return_t:ty) => {{
        let response = shared::make_request!($server, $method, $path, $payload);
        pretty_assertions::assert_eq!(response.status_code(), $status_code);

        response.json::<$return_t>()
    }};
}
#[allow(unused)]
pub(crate) use get_response;
