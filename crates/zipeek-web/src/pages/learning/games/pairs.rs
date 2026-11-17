use zipeek_core::Word;
use zipeek_exercises::cloud_match::CloudPair;

pub(super) fn make_pairs(words: Vec<Word>) -> Vec<CloudPair> {
    words
        .into_iter()
        .filter_map(|word| {
            let meaning = word
                .english
                .as_deref()?
                .split('|')
                .next()?
                .trim()
                .to_owned();
            if meaning.is_empty() {
                return None;
            }
            Some(CloudPair {
                id: word.id.as_i32(),
                hanzi: word.simplified,
                pinyin: word.pinyin,
                meaning,
            })
        })
        .collect()
}
