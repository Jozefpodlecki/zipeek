use serde_json::{Map, Value};

pub(super) const MAX_ROWS: usize = 100;
pub(super) const MAX_QUERY_BYTES: usize = 2_048;

pub(super) fn bounded_select(sql: &str) -> Result<String, &'static str> {
    let trimmed = sql.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_QUERY_BYTES {
        return Err("Enter a SELECT query of at most 2048 bytes");
    }

    let normalized = trimmed.to_ascii_lowercase();
    if !normalized.starts_with("select ") && !normalized.starts_with("select\n") {
        return Err("Only SELECT statements are supported");
    }
    if normalized.contains(';') || normalized.contains("--") || normalized.contains("/*") {
        return Err("Multiple statements and SQL comments are not supported");
    }

    let permitted = [
        " from words",
        " from information_schema.tables",
        " from pragma_database_size()",
    ];
    if !permitted.iter().any(|table| normalized.contains(table)) {
        return Err("Query the words table, information_schema.tables or pragma_database_size()");
    }

    for token in [
        "read_csv",
        "read_json",
        "read_parquet",
        "httpfs",
        "duckdb_extensions",
        "attach",
        "install",
        "load ",
        "copy ",
        "export ",
        "pragma ",
        " from read_",
    ] {
        if normalized.contains(token) {
            return Err("External I/O and administrative SQL are unavailable here");
        }
    }
    Ok(format!(
        "SELECT * FROM ({trimmed}) AS debug_preview LIMIT {MAX_ROWS}"
    ))
}

pub(super) fn as_csv(rows: &[Map<String, Value>]) -> String {
    let Some(first) = rows.first() else {
        return String::new();
    };
    let columns: Vec<_> = first.keys().cloned().collect();
    let mut csv = columns
        .iter()
        .map(|column| csv_cell(column))
        .collect::<Vec<_>>()
        .join(",");
    csv.push_str("\r\n");
    for row in rows {
        csv.push_str(
            &columns
                .iter()
                .map(|column| {
                    let raw = match row.get(column) {
                        Some(Value::String(text)) => text.clone(),
                        Some(Value::Null) | None => String::new(),
                        Some(value) => value.to_string(),
                    };
                    csv_cell(&raw)
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push_str("\r\n");
    }
    csv
}

fn csv_cell(text: &str) -> String {
    let safe = if text
        .chars()
        .next()
        .is_some_and(|first| matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r'))
    {
        format!("'{text}")
    } else {
        text.to_owned()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_bounded_dictionary_reads() {
        assert!(
            bounded_select("SELECT id, simplified FROM words LIMIT 5")
                .unwrap()
                .contains("LIMIT 100")
        );
        assert!(bounded_select("SELECT table_name FROM information_schema.tables").is_ok());
    }

    #[test]
    fn rejects_mutations_and_multiple_statements() {
        for input in [
            "DELETE FROM words",
            "SELECT * FROM words; DROP TABLE words",
            "SELECT * FROM read_csv('a')",
            "SELECT 1",
            "SELECT * FROM words -- hi",
        ] {
            assert!(bounded_select(input).is_err(), "{input}");
        }
    }

    #[test]
    fn csv_quotes_and_neutralizes_spreadsheet_formulas() {
        let row = Map::from_iter([("cell".to_owned(), Value::String("=1+1\"".into()))]);
        assert_eq!(as_csv(&[row]), "\"cell\"\r\n\"'=1+1\"\"\"\r\n");
    }
}
