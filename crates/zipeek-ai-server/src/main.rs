use std::{io, net::SocketAddr, sync::Arc};

use zipeek_ai_server::{AiServer, fixture_backend::FixtureBackend};

fn fixture_address(args: impl IntoIterator<Item = String>) -> io::Result<SocketAddr> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("--fixture") || args.next().as_deref() != Some("--bind") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Only explicit --fixture --bind <loopback-address> mode is currently supported",
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
            "Fixture mode requires one loopback bind address",
        ));
    }
    Ok(address)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = fixture_address(std::env::args().skip(1))?;
    let server = AiServer::new(Arc::new(FixtureBackend), "fixture", 2)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, server.router()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_mode_requires_explicit_loopback_bind() {
        let valid = ["--fixture", "--bind", "127.0.0.1:1420"];
        assert!(fixture_address(valid.map(str::to_owned)).is_ok());
        for args in [
            vec!["--bind", "127.0.0.1:1420"],
            vec!["--fixture", "--bind", "0.0.0.0:1420"],
            vec!["--fixture", "--bind", "127.0.0.1:1420", "extra"],
            vec!["--fixture", "--bind", "invalid"],
        ] {
            assert!(fixture_address(args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
