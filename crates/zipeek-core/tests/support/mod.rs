use zipeek_core::{
    Hsk3Level, Word, WordId,
    practice::{ExerciseVariant, ParsedPracticeQuery, PracticeFilter, fnv1a64},
    round::{CandidatePool, ContextPool, Round, RoundCandidates, build_round},
};

pub fn words() -> Vec<Word> {
    include_str!("../fixtures/round-words.tsv")
        .lines()
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            Word {
                id: WordId::try_from(fields[0].parse::<i64>().unwrap()).unwrap(),
                simplified: fields[1].to_owned(),
                pinyin: fields[2].to_owned(),
                english: Some(fields[3].to_owned()),
                traditional: None,
                pinyin_numeric: None,
                hsk2_level: None,
                hsk3_level: Some(Hsk3Level::One),
                part_of_speech: vec![],
                frequency: None,
            }
        })
        .collect()
}

pub fn pool(words: &[Word]) -> CandidatePool {
    CandidatePool {
        strict: words.to_vec(),
        hsk_widened: words.to_vec(),
        hsk_widened_any_pos: words.to_vec(),
    }
}

pub fn vectors() -> Vec<(String, Round)> {
    let words = words();
    let pool = pool(&words);
    let variants = [
        ExerciseVariant::EngPinyin,
        ExerciseVariant::PinyinHanzi,
        ExerciseVariant::HanziEng,
        ExerciseVariant::HanziToMeaning,
        ExerciseVariant::MeaningToHanzi,
        ExerciseVariant::MeaningToPinyin,
    ];
    let mut vectors = Vec::new();
    for variant in variants {
        for seeded in [false, true] {
            let mut fields = vec![("offset", "10")];
            if seeded {
                fields.push(("seed", "1"));
            }
            let query = ParsedPracticeQuery::from_pairs(fields)
                .normalize(variant, |id| words.iter().find(|word| word.id == id));
            let candidates = RoundCandidates {
                initial: &pool,
                original_seed: seeded.then_some(&words[0]),
                recovery: &[],
                recovered_views: &[],
            };
            vectors.push((
                format!(
                    "{}-{}",
                    variant.seed_token(),
                    if seeded { "seeded" } else { "offset" }
                ),
                build_round(candidates, variant, &query).unwrap(),
            ));
        }
    }
    let mut bad = words[0].clone();
    bad.id = WordId::try_from(999).unwrap();
    bad.hsk3_level = None;
    let query = ParsedPracticeQuery::from_pairs([
        ("seed", "999"),
        ("level", "3"),
        ("pos", "verb"),
        ("offset", "999"),
    ])
    .normalize(ExerciseVariant::EngPinyin, |_| Some(&bad));
    let contexts = [ContextPool {
        filter: PracticeFilter::default(),
        pool: pool.clone(),
    }];
    for (name, recovery) in [
        ("recovery-first", words.clone()),
        ("recovery-fallback", sparse_recovery(&words)),
    ] {
        let candidates = RoundCandidates {
            initial: &pool,
            original_seed: Some(&bad),
            recovery: &recovery,
            recovered_views: &contexts,
        };
        vectors.push((
            name.to_owned(),
            build_round(candidates, ExerciseVariant::EngPinyin, &query).unwrap(),
        ));
    }
    let mut recovery = sparse_recovery(&words);
    recovery[0].english = None;
    recovery[47].english = Some("one".to_owned());
    let mut tenth_words = words.clone();
    tenth_words.push(recovery[47].clone());
    let tenth_contexts = [ContextPool {
        filter: PracticeFilter::default(),
        pool: self::pool(&tenth_words),
    }];
    vectors.push((
        "recovery-tenth".to_owned(),
        build_round(
            RoundCandidates {
                initial: &pool,
                original_seed: Some(&bad),
                recovery: &recovery,
                recovered_views: &tenth_contexts,
            },
            ExerciseVariant::EngPinyin,
            &query,
        )
        .unwrap(),
    ));

    let mut missing_english = bad.clone();
    missing_english.hsk3_level = Some(Hsk3Level::One);
    missing_english.english = None;
    let english_query =
        ParsedPracticeQuery::from_pairs([("seed", "999"), ("level", "3"), ("pos", "verb")])
            .normalize(ExerciseVariant::EngPinyin, |_| Some(&missing_english));
    vectors.push((
        "recovery-missing-english".to_owned(),
        build_round(
            RoundCandidates {
                initial: &pool,
                original_seed: Some(&missing_english),
                recovery: &words,
                recovered_views: &contexts,
            },
            ExerciseVariant::EngPinyin,
            &english_query,
        )
        .unwrap(),
    ));
    let mut fallback_pool = pool.clone();
    fallback_pool.strict.truncate(8);
    for word in &mut fallback_pool.strict {
        word.english = Some("same".to_owned());
    }
    for word in &mut fallback_pool.hsk_widened[..8] {
        word.english = Some("same".to_owned());
    }
    for word in &mut fallback_pool.hsk_widened_any_pos[..8] {
        word.english = Some("same".to_owned());
    }
    let query = ParsedPracticeQuery::from_pairs([("offset", "19")])
        .normalize(ExerciseVariant::HanziEng, |_| None);
    vectors.push((
        "sliding-fallback".to_owned(),
        build_round(
            RoundCandidates {
                initial: &fallback_pool,
                original_seed: None,
                recovery: &[],
                recovered_views: &[],
            },
            ExerciseVariant::HanziEng,
            &query,
        )
        .unwrap(),
    ));
    let mut draw_words = words.clone();
    for (index, word) in draw_words[4..12].iter_mut().enumerate() {
        word.english = Some(format!("reserve {}", index % 3));
    }
    let mut draw_pool = self::pool(&draw_words);
    draw_pool.strict.truncate(12);
    let draw_query = ParsedPracticeQuery::default().normalize(ExerciseVariant::HanziEng, |_| None);
    vectors.push((
        "distractor-fallback".to_owned(),
        build_round(
            RoundCandidates {
                initial: &draw_pool,
                original_seed: None,
                recovery: &[],
                recovered_views: &[],
            },
            ExerciseVariant::HanziEng,
            &draw_query,
        )
        .unwrap(),
    ));
    vectors
}

fn sparse_recovery(words: &[Word]) -> Vec<Word> {
    (1..=128)
        .map(|value| {
            let mut word = words[0].clone();
            word.id = WordId::try_from(value).unwrap();
            if value != 1 {
                word.english = None;
            }
            word
        })
        .collect()
}

pub fn report() -> String {
    vectors()
        .into_iter()
        .map(|(name, round)| {
            format!(
                "{name}\t{}\t{:016x}\t{round:?}\n",
                round.canonical_input,
                fnv1a64(&round.canonical_input)
            )
        })
        .collect()
}
