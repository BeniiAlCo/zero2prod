#[tokio::main]
async fn main() -> hyper::Result<()> {
    zero2prod::run("127.0.0.1:8000")?.await
}
