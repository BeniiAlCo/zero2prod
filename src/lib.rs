use axum::{http::StatusCode, routing::get, routing::IntoMakeService, Router, Server};

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn run(
    address: &str,
) -> hyper::Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>> {
    let app = Router::new().route("/health_check", get(health_check));

    let server = Server::bind(&address.parse().unwrap()).serve(app.into_make_service());

    Ok(server)
}
