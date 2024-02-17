use axum::response::Response;

pub fn internal_server_error<T: Into<String>>(message: T) -> Response {
    Response::builder()
        .status(500)
        .body(message.into().into())
        .unwrap()
}

pub fn bad_request<T: Into<String>>(message: T) -> Response {
    Response::builder()
        .status(400)
        .body(message.into().into())
        .unwrap()
}
