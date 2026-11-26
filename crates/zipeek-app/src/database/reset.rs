use std::{fs, io::ErrorKind, path::Path};

use super::NativeConnections;

pub(super) fn reset_personal_database(
    connections: &mut Option<NativeConnections>,
    personal: Option<&Path>,
) -> Result<(), String> {
    let path = personal.ok_or("Native personal storage path is unavailable")?;
    if path.file_name().and_then(|name| name.to_str()) != Some("zipeek-personal.duckdb") {
        return Err("Refusing to reset an unexpected native database path".into());
    }
    connections.take();
    let wal = path.with_file_name("zipeek-personal.duckdb.wal");
    for file in [path, wal.as_path()] {
        match fs::symlink_metadata(file) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(
                    "Refusing to clear a native personal database symlink or directory".into(),
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Cannot inspect native personal data: {error}")),
        }
    }
    for file in [path, wal.as_path()] {
        match fs::remove_file(file) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Cannot clear native personal data: {error}")),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{memory_connection, open_personal_connection};
    use zipeek_db::native::Parameters;

    fn fixture() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "zipeek-reset-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reset_closes_personal_database_removes_it_and_preserves_dictionary() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        let path = root.join("zipeek-personal.duckdb");
        let dictionary = root.join("hanzi.duckdb");
        fs::write(&dictionary, b"dictionary fixture").unwrap();
        let personal = open_personal_connection(&path).unwrap();
        personal
            .execute(
                "INSERT INTO personal.practice_sessions VALUES ('test', 'private')",
                Parameters::None,
            )
            .unwrap();
        let mut connections = Some(NativeConnections {
            dictionary: memory_connection().unwrap(),
            personal: Some(personal),
        });
        reset_personal_database(&mut connections, Some(&path)).unwrap();
        assert!(connections.is_none());
        assert!(!path.exists());
        assert!(!root.join("zipeek-personal.duckdb.wal").exists());
        assert_eq!(fs::read(&dictionary).unwrap(), b"dictionary fixture");
        reset_personal_database(&mut connections, Some(&path)).unwrap();
        let reopened = open_personal_connection(&path).unwrap();
        assert!(
            super::super::execute_json(&reopened, "SELECT * FROM personal.practice_sessions", &[])
                .unwrap()
                .is_empty()
        );
        drop(reopened);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reset_rejects_unexpected_paths_and_reports_removal_failures() {
        let root = fixture();
        let path = root.join("zipeek-personal.duckdb");
        fs::create_dir_all(&path).unwrap();
        assert!(reset_personal_database(&mut None, Some(&root.join("hanzi.duckdb"))).is_err());
        assert!(reset_personal_database(&mut None, None).is_err());
        assert!(reset_personal_database(&mut None, Some(&path)).is_err());
        assert!(path.is_dir());
        fs::remove_dir_all(root).unwrap();
    }
}
