use std::{io, net::SocketAddr, sync::Arc};

use zipeek_ai_server::{
    AiServer,
    llama_cpp::{LocalLlama, LocalModelConfig},
};

fn bind_address(args: impl IntoIterator<Item = String>) -> io::Result<SocketAddr> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("--bind") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Expected --bind <loopback-address>",
        ));
    }
    let address: SocketAddr = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing bind address"))?
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid bind address"))?;
    if !address.ip().is_loopback() || args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Expected one loopback address",
        ));
    }
    Ok(address)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = bind_address(std::env::args().skip(1))?;
    let config = LocalModelConfig::from_environment()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let backend = LocalLlama::load(config).await.map_err(io::Error::other)?;
    let server = AiServer::new(Arc::new(backend), "local-gguf", 1)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, server.router()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_explicit_loopback() {
        assert!(bind_address(["--bind", "127.0.0.1:1430"].map(str::to_owned)).is_ok());
        assert!(bind_address(["--bind", "0.0.0.0:1430"].map(str::to_owned)).is_err());
        assert!(
            bind_address(["--fixture", "--bind", "127.0.0.1:1430"].map(str::to_owned)).is_err()
        );
    }
}
