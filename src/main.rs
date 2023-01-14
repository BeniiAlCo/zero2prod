use std::net::TcpListener;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    zero2prod::run(TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port 8000"))?.await
}
