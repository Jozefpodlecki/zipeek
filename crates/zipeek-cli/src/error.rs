use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum CliError {
    Workspace(io::Error),
    Command {
        name: &'static str,
        source: Box<dyn Error + Send + Sync>,
    },
    ReleaseDiscovery {
        downloads: PathBuf,
        source: Box<dyn Error + Send + Sync>,
    },
}

impl CliError {
    pub fn command(name: &'static str, source: Box<dyn Error + Send + Sync>) -> Self {
        Self::Command { name, source }
    }

    pub fn release_discovery(downloads: PathBuf, source: Box<dyn Error + Send + Sync>) -> Self {
        Self::ReleaseDiscovery { downloads, source }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Workspace(_) => f.write_str("could not locate the workspace root"),
            Self::Command { name, .. } => write!(f, "{name} failed"),
            Self::ReleaseDiscovery { downloads, .. } => {
                write!(
                    f,
                    "could not select a source release in {}",
                    downloads.display()
                )
            }
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Workspace(error) => Some(error),
            Self::Command { source, .. } | Self::ReleaseDiscovery { source, .. } => {
                Some(source.as_ref())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_keep_operation_context_and_original_cause() {
        let error = CliError::command(
            "db-install",
            io::Error::new(io::ErrorKind::NotFound, "missing bundle").into(),
        );
        assert_eq!(error.to_string(), "db-install failed");
        assert_eq!(error.source().unwrap().to_string(), "missing bundle");

        let error = CliError::release_discovery(
            PathBuf::from("Downloads"),
            io::Error::new(io::ErrorKind::NotFound, "not found").into(),
        );
        assert!(error.to_string().contains("Downloads"));
        assert_eq!(error.source().unwrap().to_string(), "not found");
    }
}
