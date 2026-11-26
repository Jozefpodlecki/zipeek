use super::super::DbError;

pub(super) fn parse_session_count(value: Option<&str>) -> Result<u64, DbError> {
    value
        .and_then(|count| count.parse::<u64>().ok())
        .ok_or_else(|| DbError::Decode("Invalid stored Practice session count".into()))
}

pub(super) fn valid_identifier(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}
