pub(super) struct SqlExample {
    pub label: &'static str,
    pub sql: &'static str,
}

pub(super) const EXAMPLES: [SqlExample; 20] = [
    SqlExample {
        label: "First 10 words",
        sql: "SELECT id, simplified, pinyin FROM words ORDER BY id LIMIT 10",
    },
    SqlExample {
        label: "Word count",
        sql: "SELECT count(*) AS word_count FROM words",
    },
    SqlExample {
        label: "Simplified and traditional",
        sql: "SELECT simplified, traditional, pinyin FROM words WHERE traditional IS NOT NULL LIMIT 20",
    },
    SqlExample {
        label: "Search by Hanzi",
        sql: "SELECT simplified, pinyin, english FROM words WHERE simplified LIKE '%学%' LIMIT 20",
    },
    SqlExample {
        label: "Search English meanings",
        sql: "SELECT simplified, pinyin, english FROM words WHERE english ILIKE '%learn%' LIMIT 20",
    },
    SqlExample {
        label: "HSK 2.0 level 1",
        sql: "SELECT simplified, pinyin, english FROM words WHERE hsk2_level = 1 LIMIT 20",
    },
    SqlExample {
        label: "HSK 3.0 level 1",
        sql: "SELECT simplified, pinyin, english FROM words WHERE hsk3_level = 1 LIMIT 20",
    },
    SqlExample {
        label: "HSK 2.0 counts",
        sql: "SELECT hsk2_level, count(*) AS words FROM words WHERE hsk2_level IS NOT NULL GROUP BY hsk2_level ORDER BY hsk2_level",
    },
    SqlExample {
        label: "HSK 3.0 counts",
        sql: "SELECT hsk3_level, count(*) AS words FROM words WHERE hsk3_level IS NOT NULL GROUP BY hsk3_level ORDER BY hsk3_level",
    },
    SqlExample {
        label: "Words without HSK",
        sql: "SELECT simplified, pinyin, english FROM words WHERE hsk2_level IS NULL AND hsk3_level IS NULL LIMIT 20",
    },
    SqlExample {
        label: "Shortest words",
        sql: "SELECT simplified, pinyin, english FROM words ORDER BY length(simplified), id LIMIT 20",
    },
    SqlExample {
        label: "Longest words",
        sql: "SELECT simplified, pinyin, english FROM words ORDER BY length(simplified) DESC, id LIMIT 20",
    },
    SqlExample {
        label: "One-character entries",
        sql: "SELECT simplified, pinyin, english FROM words WHERE length(simplified) = 1 LIMIT 20",
    },
    SqlExample {
        label: "Four-character entries",
        sql: "SELECT simplified, pinyin, english FROM words WHERE length(simplified) = 4 LIMIT 20",
    },
    SqlExample {
        label: "Common frequency",
        sql: "SELECT simplified, pinyin, frequency FROM words WHERE frequency IS NOT NULL ORDER BY frequency ASC LIMIT 20",
    },
    SqlExample {
        label: "Part-of-speech counts",
        sql: "SELECT trim(pos.label) AS part_of_speech, count(*) AS words FROM words CROSS JOIN UNNEST(string_split(part_of_speech, ',')) AS pos(label) WHERE trim(pos.label) <> '' GROUP BY trim(pos.label) ORDER BY words DESC LIMIT 20",
    },
    SqlExample {
        label: "Duplicate Hanzi readings",
        sql: "SELECT simplified, count(*) AS readings FROM words GROUP BY simplified HAVING count(*) > 1 ORDER BY readings DESC LIMIT 20",
    },
    SqlExample {
        label: "Missing English glosses",
        sql: "SELECT simplified, pinyin FROM words WHERE english IS NULL LIMIT 20",
    },
    SqlExample {
        label: "Dictionary tables",
        sql: "SELECT table_name FROM information_schema.tables ORDER BY table_name LIMIT 50",
    },
    SqlExample {
        label: "Database storage",
        sql: "SELECT database_size, block_size, total_blocks, used_blocks, wal_size FROM pragma_database_size()",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::system::debug::sql::bounded_select;

    #[test]
    fn examples_are_unique_and_bounded() {
        let mut labels = std::collections::BTreeSet::new();
        assert_eq!(EXAMPLES.len(), 20);
        for example in EXAMPLES {
            assert!(labels.insert(example.label));
            assert!(bounded_select(example.sql).is_ok(), "{}", example.label);
        }
    }
}
