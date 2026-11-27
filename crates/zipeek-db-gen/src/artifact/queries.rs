pub(super) const TABLES: &str = r#"
SELECT table_name
FROM duckdb_tables()
WHERE database_name = current_database()
  AND schema_name = 'main'
  AND NOT internal
ORDER BY table_name
"#;

pub(super) const ROWS: &str = r#"
SELECT to_json(words)::VARCHAR
FROM words
ORDER BY id
"#;

pub(super) const SEARCH_DOCUMENTS: &str = r#"
SELECT to_json(word_search_documents)::VARCHAR
FROM word_search_documents
ORDER BY word_id
"#;

pub(super) const SEARCH_KEYS: &str = r#"
SELECT to_json(word_search_keys)::VARCHAR
FROM word_search_keys
ORDER BY word_id
"#;

pub(super) const DICTIONARY_VERSIONS: &str = r#"
SELECT to_json(dictionary_versions)::VARCHAR
FROM dictionary_versions
ORDER BY snapshot_fingerprint
"#;

pub(super) const VOCABULARY_IDENTITIES: &str = r#"
SELECT to_json(vocabulary_identities)::VARCHAR
FROM vocabulary_identities
ORDER BY word_id
"#;

pub(super) const CHARACTERS: &str = r#"
SELECT to_json(characters)::VARCHAR
FROM characters
ORDER BY unicode_scalar
"#;

pub(super) const PINYIN_SYLLABLE_CANDIDATES: &str = r#"
SELECT to_json(pinyin_syllable_candidates)::VARCHAR
FROM pinyin_syllable_candidates
ORDER BY pinyin_numeric
"#;

pub(super) const CHARACTER_GLOSS_CANDIDATES: &str = r#"
SELECT to_json(character_gloss_candidates)::VARCHAR
FROM character_gloss_candidates
ORDER BY character, source_word_id, sense_order
"#;

pub(super) const CHARACTER_READINGS: &str = r#"
SELECT to_json(character_readings)::VARCHAR
FROM character_readings
ORDER BY character, pinyin_numeric
"#;

pub(super) const WORD_RELATIONS: &str = r#"
SELECT to_json(word_relations)::VARCHAR
FROM word_relations
ORDER BY relation_id
"#;

pub(super) const WORD_CHARACTERS: &str = r#"
SELECT to_json(word_characters)::VARCHAR
FROM word_characters
ORDER BY word_id, character_position
"#;

pub(super) const HSK_MEMBERSHIPS: &str = r#"
SELECT to_json(struct_pack(word_id := word_id, system := system, level := level))::VARCHAR
FROM word_hsk_membership
ORDER BY word_id, system
"#;

pub(super) const POS_MEMBERSHIPS: &str = r#"
SELECT to_json(struct_pack(word_id := word_id, pos := pos))::VARCHAR
FROM word_pos_membership
ORDER BY word_id, pos
"#;

pub(super) const SENSES: &str = r#"
SELECT to_json(word_senses)::VARCHAR
FROM word_senses
ORDER BY word_id, sense_order
"#;

pub(super) const SENSE_DISPLAY: &str = r#"
SELECT to_json(sense_display)::VARCHAR
FROM sense_display
ORDER BY word_id, sense_order
"#;

pub(super) const SENSE_QUALIFIERS: &str = r#"
SELECT to_json(sense_qualifiers)::VARCHAR
FROM sense_qualifiers
ORDER BY word_id, sense_order, qualifier_kind, qualifier_code
"#;

pub(super) const SENSE_RELATIONS: &str = r#"
SELECT to_json(sense_relations)::VARCHAR
FROM sense_relations
ORDER BY word_id, sense_order, relation_order
"#;

pub(super) const CATEGORIES: &str = r#"
SELECT to_json(categories)::VARCHAR
FROM categories
ORDER BY slug
"#;

pub(super) const CLASSIFICATION_RUNS: &str = r#"
SELECT to_json(classification_runs)::VARCHAR
FROM classification_runs
ORDER BY id
"#;

pub(super) const SENSE_POS: &str = r#"
SELECT to_json(sense_pos)::VARCHAR
FROM sense_pos
ORDER BY word_id, sense_order, pos_code
"#;

pub(super) const SENSE_CATEGORIES: &str = r#"
SELECT to_json(sense_categories)::VARCHAR
FROM sense_categories
ORDER BY word_id, sense_order, category_slug
"#;

pub(super) const COLUMNS: &str = r#"
SELECT to_json(struct_pack(name := column_name, type := data_type, nullable := is_nullable, position := ordinal_position))::VARCHAR
FROM information_schema.columns
WHERE table_catalog = current_database()
  AND table_name = 'words'
ORDER BY ordinal_position
"#;

pub(super) const CONSTRAINTS: &str = r#"
SELECT to_json(struct_pack(kind := constraint_type, definition := constraint_text, columns := constraint_column_names))::VARCHAR
FROM duckdb_constraints()
WHERE database_name = current_database()
  AND table_name = 'words'
ORDER BY constraint_type, constraint_text
"#;

pub(super) const INDEXES: &str = r#"
SELECT to_json(struct_pack(name := index_name, unique_index := is_unique, primary_index := is_primary, expressions := expressions))::VARCHAR
FROM duckdb_indexes()
WHERE database_name = current_database()
  AND table_name = 'words'
ORDER BY index_name
"#;

