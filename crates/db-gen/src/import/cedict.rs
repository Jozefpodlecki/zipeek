use std::collections::BTreeMap;

use zipeek_core::normalize_for_comparison;

use super::ImportError;

#[derive(Debug)]
pub(super) struct CedictReading {
    pub line: usize,
    pub glosses: Vec<String>,
}

#[derive(Debug)]
pub struct CedictIndex {
    readings: BTreeMap<(String, String), Vec<CedictReading>>,
    entry_count: usize,
}

impl CedictIndex {
    pub fn parse(bytes: &[u8]) -> Result<Self, ImportError> {
        let text = std::str::from_utf8(bytes).map_err(ImportError::CedictUtf8)?;
        let mut readings: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut entry_count = 0;
        for (index, original) in text.lines().enumerate() {
            let line = original.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let number = index + 1;
            let invalid = |reason| ImportError::CedictLine {
                line: number,
                original: original.to_owned(),
                reason,
            };
            let (headwords, remainder) = line
                .split_once('[')
                .ok_or_else(|| invalid("missing opening pinyin bracket"))?;
            let (pinyin, meanings) = remainder
                .split_once(']')
                .ok_or_else(|| invalid("missing closing pinyin bracket"))?;
            let headwords: Vec<_> = headwords.split_whitespace().collect();
            if headwords.len() != 2 || pinyin.trim().is_empty() {
                return Err(invalid(
                    "expected traditional, simplified and nonempty pinyin",
                ));
            }
            let meanings = meanings
                .trim()
                .strip_prefix('/')
                .and_then(|text| text.strip_suffix('/'))
                .ok_or_else(|| invalid("meanings must be enclosed in slashes"))?;
            let glosses = meanings
                .split('/')
                .map(str::trim)
                .filter(|gloss| !gloss.is_empty())
                .map(str::to_owned)
                .collect();
            let key = (headwords[1].to_owned(), normalize_for_comparison(pinyin));
            readings.entry(key).or_default().push(CedictReading {
                line: number,
                glosses,
            });
            entry_count += 1;
        }
        Ok(Self {
            readings,
            entry_count,
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub(super) fn lookup(&self, simplified: &str, pinyin: &str) -> &[CedictReading] {
        self.readings
            .get(&(simplified.to_owned(), normalize_for_comparison(pinyin)))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}
