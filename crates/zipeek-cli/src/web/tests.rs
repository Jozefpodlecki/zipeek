use super::*;

#[test]
fn arrow_rewrite_is_exact() {
    assert_eq!(
        rewrite_arrow("import*as a from\"apache-arrow\";").unwrap(),
        "import*as a from\"./arrow.mjs\";"
    );
    assert!(rewrite_arrow("import foo from './other.js'").is_err());
    assert_eq!(
        rewrite_arrow("from\"apache-arrow\"; from\"apache-arrow\"").unwrap(),
        "from\"./arrow.mjs\"; from\"./arrow.mjs\"",
    );
}

#[test]
fn publish_replaces_old_assets_after_staging() -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zipeek-web-publish-{}-{unique}",
        std::process::id()
    ));
    let current = root.join("duckdb");
    let stage = root.join("duckdb.zipeek-staging");
    fs::create_dir_all(&current)?;
    fs::create_dir_all(&stage)?;
    fs::write(current.join("old"), "old")?;
    fs::write(stage.join("new"), "new")?;
    let result = (|| -> Result<()> {
        publish(&stage, &current)?;
        assert!(!current.join("old").exists());
        assert_eq!(fs::read_to_string(current.join("new"))?, "new");
        assert!(!stage.exists());
        assert!(!root.join("duckdb.zipeek-previous").exists());
        Ok(())
    })();
    fs::remove_dir_all(root)?;
    result
}

#[test]
fn failed_staged_preparation_does_not_change_published_assets() -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root =
        std::env::temp_dir().join(format!("zipeek-web-stage-{}-{unique}", std::process::id()));
    let current = root.join("summary");
    fs::create_dir_all(&current)?;
    fs::write(current.join("summary.mjs"), "existing")?;
    let stage = root.join("summary.zipeek-staging");
    fs::create_dir_all(&stage)?;
    fs::write(stage.join("summary.mjs"), "incomplete")?;
    assert_eq!(fs::read_to_string(current.join("summary.mjs"))?, "existing");
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn asset_manifests_have_distinct_entries() {
    assert!(DUCKDB_FILES.contains(&"duckdb-base.wasm"));
    assert!(SUMMARY_FILES.contains(&"summary.mjs"));
    assert!(!SUMMARY_FILES.contains(&"duckdb-base.wasm"));
}
