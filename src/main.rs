#[tokio::main]
async fn main() -> hyper::Result<()> {
    zero2prod::run()?.await
}
