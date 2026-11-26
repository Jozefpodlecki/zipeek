pub const RESET_HASH: &str = "#/reset-app";
pub const CONFIRMATION_KEY: &str = "zipeek.reset.confirmed";

pub fn owns_storage_key(key: &str) -> bool {
    key.starts_with("zipeek.") || key == "RUST_LOG"
}

pub fn owns_private_entry(name: &str) -> bool {
    name.starts_with("zipeek-") || name.starts_with("zipeek.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_covers_current_personal_storage_without_other_apps() {
        for key in [
            "zipeek.preferences.v1",
            "zipeek.user_word_sets.v1",
            "zipeek.courses.progress.v1",
            "zipeek.crosswords.v5.session",
            "zipeek.practice.session.v1.session",
            "RUST_LOG",
        ] {
            assert!(owns_storage_key(key));
        }
        for key in ["another-app.preferences", "zipeekish.settings", "theme", ""] {
            assert!(!owns_storage_key(key));
        }
        for name in [
            "zipeek-personal-learning-v1.duckdb",
            "zipeek-personal-learning-v1.duckdb.wal",
            "zipeek-personal-r119-probe.duckdb",
            "zipeek-learning-snapshot-v1.json",
            "zipeek-opfs-probe-sample.tmp",
        ] {
            assert!(owns_private_entry(name));
        }
        for name in [
            "hanzi-20261124-214934.duckdb",
            "model.safetensors",
            "other-app.duckdb",
            "duckdb",
        ] {
            assert!(!owns_private_entry(name));
        }
    }
}
