use zipeek_static_server::{parse_args, serve};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args(std::env::args().skip(1))?;
    serve(config).await
}
