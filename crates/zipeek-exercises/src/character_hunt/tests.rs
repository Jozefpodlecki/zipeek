use super::*;

fn vocabulary() -> Vec<Candidate> {
    [
        "中国", "学生", "天气", "工作", "音乐", "旅游", "电脑", "朋友", "老师", "开心", "美食",
        "图书", "运动", "环境", "文化", "知识", "语言", "学校", "爱情", "生命", "发展", "科技",
        "家庭", "经济", "社会", "历史", "教育", "健康",
    ]
    .into_iter()
    .map(|hanzi| Candidate {
        hanzi: hanzi.to_owned(),
        readings: None,
        meaning: None,
    })
    .collect()
}

#[test]
fn difficulty_has_approved_board_sizes() {
    assert_eq!(Difficulty::ALL.map(Difficulty::size), [2, 3, 4, 5]);
    assert_eq!(Difficulty::default(), Difficulty::Medium);
    assert_eq!(Difficulty::parse("hardcore"), Some(Difficulty::Hardcore));
}

#[test]
fn generated_boards_are_deterministic_and_solvable() {
    for difficulty in Difficulty::ALL {
        let mut game = CharacterHunt::generate(&vocabulary(), difficulty, 42).unwrap();
        assert_eq!(game.tiles.len(), difficulty.size().pow(2));
        assert_eq!(
            game,
            CharacterHunt::generate(&vocabulary(), difficulty, 42).unwrap()
        );
        let solutions = game.answers.clone();
        for solution in &solutions {
            for index in &solution.indices {
                assert_ne!(game.tap(*index).unwrap(), TapResult::Incorrect);
            }
        }
        assert!(game.complete());
        assert_eq!(game.found.len(), game.total_words());
        assert!(game.tiles.iter().all(|tile| tile.completed || tile.filler));
    }
}

#[test]
fn three_character_dictionary_words_can_be_found_in_order() {
    let mut candidates = vocabulary();
    candidates.push(Candidate {
        hanzi: "图书馆".to_owned(),
        readings: None,
        meaning: None,
    });
    let mut game = CharacterHunt::generate(&candidates, Difficulty::Medium, 4).unwrap();
    let triple = game
        .answers
        .iter()
        .find(|answer| answer.word.chars().count() == 3)
        .unwrap()
        .clone();
    assert_eq!(game.tap(triple.indices[0]), Ok(TapResult::Selected));
    assert_eq!(game.tap(triple.indices[1]), Ok(TapResult::Selected));
    assert_eq!(game.tap(triple.indices[2]), Ok(TapResult::Found));
    assert!(
        triple
            .indices
            .iter()
            .all(|index| game.tiles[*index].completed)
    );
}

#[test]
fn hardcore_never_invents_filler_characters() {
    let words = [
        "中国", "学生", "天气", "工作", "音乐", "旅游", "电脑", "朋友", "老师", "开心", "美食",
        "图书",
    ];
    let candidates: Vec<Candidate> = words
        .into_iter()
        .map(|hanzi| Candidate {
            hanzi: hanzi.to_owned(),
            readings: None,
            meaning: None,
        })
        .collect();
    assert!(matches!(
        CharacterHunt::generate(&candidates, Difficulty::Hardcore, 17),
        Err(HuntError::InsufficientVocabulary { .. }),
    ));
}

#[test]
fn retries_alternative_word_combinations_before_reporting_insufficient_vocabulary() {
    let words = ["甲乙", "甲丙", "乙丁"];
    let candidates: Vec<Candidate> = words
        .into_iter()
        .map(|hanzi| Candidate {
            hanzi: hanzi.to_owned(),
            readings: None,
            meaning: None,
        })
        .collect();
    for seed in 0..64 {
        let mut game = CharacterHunt::generate(&candidates, Difficulty::Easy, seed)
            .expect("an alternative complete word combination exists");
        assert_eq!(game.total_words(), 2);
        assert_eq!(game.tiles.len(), 4);
        for answer in game.answers.clone() {
            for index in answer.indices {
                assert_ne!(game.tap(index), Ok(TapResult::Incorrect));
            }
        }
        assert!(game.complete());
    }
}

#[test]
fn round_stats_only_include_completed_boards_and_do_not_double_count() {
    let mut game = CharacterHunt::generate(&vocabulary(), Difficulty::Easy, 31).unwrap();
    let stats = HuntSessionStats::default();
    assert_eq!(stats.with_completed_round(&game), None);
    let answer = game.answers[0].clone();
    game.tap(answer.indices[0]).unwrap();
    game.clear_selection();
    assert!(game.selected.is_empty());
    assert_eq!(game.mistakes, 0);
    for answer in game.answers.clone() {
        for index in answer.indices {
            game.tap(index).unwrap();
        }
    }
    let finished = stats.with_completed_round(&game).unwrap();
    assert_eq!(finished.completed_rounds, 1);
    assert_eq!(finished.found_words, game.total_words());
    assert_eq!(finished.mistakes, 0);
}

#[test]
fn invalid_selection_does_not_consume_tiles() {
    let mut game = CharacterHunt::generate(&vocabulary(), Difficulty::Easy, 7).unwrap();
    assert_eq!(game.tap(usize::MAX), Err(HuntError::InvalidTile));
    let answer = game.answers[0].clone();
    let wrong = (0..game.tiles.len())
        .find(|index| !answer.indices.contains(index))
        .unwrap();
    assert_eq!(game.tap(answer.indices[0]).unwrap(), TapResult::Selected);
    let result = game.tap(wrong).unwrap();
    assert_eq!(result, TapResult::Incorrect);
    assert_eq!(game.mistakes, 1);
    assert!(game.selected.is_empty());
    assert!(game.found.is_empty());
}

#[test]
fn no_fake_puzzles_when_vocabulary_is_insufficient() {
    let candidates = vec![Candidate {
        hanzi: "中国".to_owned(),
        readings: None,
        meaning: None,
    }];
    assert!(matches!(
        CharacterHunt::generate(&candidates, Difficulty::Hardcore, 0),
        Err(HuntError::InsufficientVocabulary { .. })
    ));
}

#[test]
fn only_real_chinese_two_or_three_character_entries_are_used() {
    let mut candidates = vocabulary();
    candidates.extend(["A12", "你的朋友们", "中中"].map(|hanzi| Candidate {
        hanzi: hanzi.to_owned(),
        readings: None,
        meaning: None,
    }));
    let game = CharacterHunt::generate(&candidates, Difficulty::Medium, 2).unwrap();
    assert!(
        game.answers
            .iter()
            .all(|answer| (2..=3).contains(&answer.word.chars().count()))
    );
    assert!(!game.answers.iter().any(|answer| answer.word == "中中"));
}

#[test]
fn dictionary_readings_follow_tiles_after_shuffling() {
    let candidates = vec![
        Candidate {
            hanzi: "中国".to_owned(),
            readings: Some(vec!["zhōng".to_owned(), "guó".to_owned()]),
            meaning: None,
        },
        Candidate {
            hanzi: "天气".to_owned(),
            readings: Some(vec!["tiān".to_owned(), "qì".to_owned()]),
            meaning: None,
        },
    ];
    let board = CharacterHunt::generate(&candidates, Difficulty::Easy, 42).unwrap();
    for tile in &board.tiles {
        let expected = match tile.character {
            '中' => "zhōng",
            '国' => "guó",
            '天' => "tiān",
            '气' => "qì",
            _ => panic!("Unexpected character in the fixture"),
        };
        assert_eq!(tile.pinyin.as_deref(), Some(expected));
    }
}

#[test]
fn hint_cells_are_real_word_positions_and_reveal_is_assisted() {
    let mut board = CharacterHunt::generate(&vocabulary(), Difficulty::Easy, 7).unwrap();
    let indices = board
        .hint_indices()
        .expect("the board has an unfinished word");
    assert!(indices.len() == 2 || indices.len() == 3);
    assert!(indices.iter().all(|index| *index < board.tiles.len()));
    let word = board.reveal_word().expect("the hint can be revealed");
    assert!(board.found.contains(&word));
    assert!(indices.iter().all(|index| board.tiles[*index].completed));
    assert_eq!(board.mistakes, 0);
    assert_ne!(board.hint_indices(), Some(indices));
}

#[test]
fn ambiguous_dictionary_pronunciations_are_not_presented_as_certain() {
    let candidates = vec![
        Candidate {
            hanzi: "中国".to_owned(),
            readings: Some(vec!["zhōng".to_owned(), "guó".to_owned()]),
            meaning: None,
        },
        Candidate {
            hanzi: "中国".to_owned(),
            readings: Some(vec!["zhòng".to_owned(), "guó".to_owned()]),
            meaning: None,
        },
        Candidate {
            hanzi: "天气".to_owned(),
            readings: Some(vec!["tiān".to_owned(), "qì".to_owned()]),
            meaning: None,
        },
    ];
    let board = CharacterHunt::generate(&candidates, Difficulty::Easy, 7).unwrap();
    for tile in &board.tiles {
        if tile.character == '中' || tile.character == '国' {
            assert_eq!(tile.pinyin, None);
        } else {
            assert!(tile.pinyin.is_some());
        }
    }
}

#[test]
fn completed_words_retain_dictionary_definitions_after_shuffle() {
    let candidates = [
        Candidate {
            hanzi: "中国".to_owned(),
            readings: None,
            meaning: Some("China".to_owned()),
        },
        Candidate {
            hanzi: "天气".to_owned(),
            readings: None,
            meaning: Some("weather".to_owned()),
        },
    ];
    let mut board = CharacterHunt::generate(&candidates, Difficulty::Easy, 42).unwrap();
    assert!(board.completed_words().is_empty());
    for answer in board.answers.clone() {
        for index in answer.indices {
            board.tap(index).unwrap();
        }
    }
    assert_eq!(
        board.completed_words(),
        vec![
            FoundWord {
                hanzi: "中国".to_owned(),
                pinyin: None,
                meaning: Some("China".to_owned()),
            },
            FoundWord {
                hanzi: "天气".to_owned(),
                pinyin: None,
                meaning: Some("weather".to_owned()),
            },
        ],
    );
}

#[test]
fn multiple_dictionary_glosses_are_preserved_without_guessing() {
    let candidates = [
        Candidate {
            hanzi: "中国".to_owned(),
            readings: None,
            meaning: Some("China".to_owned()),
        },
        Candidate {
            hanzi: "中国".to_owned(),
            readings: None,
            meaning: Some("Middle Kingdom".to_owned()),
        },
        Candidate {
            hanzi: "天气".to_owned(),
            readings: None,
            meaning: Some("weather".to_owned()),
        },
    ];
    let mut board = CharacterHunt::generate(&candidates, Difficulty::Easy, 42).unwrap();
    for answer in board.answers.clone() {
        for index in answer.indices {
            board.tap(index).unwrap();
        }
    }
    let found = board.completed_words();
    assert!(
        found.iter().any(|word| word.hanzi == "中国"
            && word.meaning.as_deref() == Some("China · Middle Kingdom"))
    );
    assert!(
        found
            .iter()
            .any(|word| word.hanzi == "天气" && word.meaning.as_deref() == Some("weather"))
    );
}

#[test]
fn completed_word_reading_keeps_original_syllable_order_after_shuffle() {
    let candidates = [
        Candidate {
            hanzi: "中国".to_owned(),
            readings: Some(vec!["zhōng".to_owned(), "guó".to_owned()]),
            meaning: Some("China".to_owned()),
        },
        Candidate {
            hanzi: "天气".to_owned(),
            readings: Some(vec!["tiān".to_owned(), "qì".to_owned()]),
            meaning: Some("weather".to_owned()),
        },
    ];
    let mut board = CharacterHunt::generate(&candidates, Difficulty::Easy, 17).unwrap();
    for answer in board.answers.clone() {
        for index in answer.indices {
            board.tap(index).unwrap();
        }
    }
    let words = board.completed_words();
    assert_eq!(words.len(), 2);
    assert!(
        words
            .iter()
            .any(|word| word.hanzi == "中国" && word.pinyin.as_deref() == Some("zhōng guó"))
    );
    assert!(
        words
            .iter()
            .any(|word| word.hanzi == "天气" && word.pinyin.as_deref() == Some("tiān qì"))
    );
}

#[test]
fn candidate_eligibility_matches_board_generation_constraints() {
    for (word, eligible) in [
        ("中国", true),
        (" 中国 ", true),
        ("图书馆", true),
        ("人人", false),
        ("A中", false),
        ("中", false),
        ("中文学校", false),
    ] {
        let candidate = Candidate {
            hanzi: word.into(),
            readings: None,
            meaning: None,
        };
        assert_eq!(candidate.is_eligible(), eligible, "{word}");
    }
    let repeated = Candidate {
        hanzi: "人人".into(),
        readings: None,
        meaning: None,
    };
    assert!(matches!(
        CharacterHunt::generate(&[repeated], Difficulty::Easy, 1),
        Err(HuntError::InsufficientVocabulary { .. })
    ));
}

#[test]
fn invalid_taps_do_not_change_board_or_add_mistakes() {
    let mut board = CharacterHunt::generate(&vocabulary(), Difficulty::Easy, 31).unwrap();
    let before = board.clone();
    assert_eq!(board.tap(usize::MAX), Err(HuntError::InvalidTile));
    assert_eq!(board, before);
    assert!(!board.complete());
}
