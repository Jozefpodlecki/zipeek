use super::*;
use zipeek_core::WordId;

fn word(id: i64, hanzi: &str, english: &str) -> Word {
    Word {
        id: WordId::try_from(id).unwrap(),
        simplified: hanzi.into(),
        traditional: None,
        pinyin: "shēngchǎn".into(),
        pinyin_numeric: None,
        english: Some(english.into()),
        part_of_speech: Vec::new(),
        hsk2_level: None,
        hsk3_level: None,
        frequency: None,
    }
}

#[test]
fn each_pair_has_one_chinese_and_one_english_tile() {
    let tiles = mosaic_tiles(
        &[word(1, "生产", "to produce"), word(2, "发展", "develop")],
        3,
    );
    assert_eq!(tiles.len(), 4);
    for pair in 0..2 {
        assert_eq!(
            tiles
                .iter()
                .filter(|tile| tile.pair == pair && tile.is_english)
                .count(),
            1
        );
        assert_eq!(
            tiles
                .iter()
                .filter(|tile| tile.pair == pair && !tile.is_english)
                .count(),
            1
        );
    }
}

#[test]
fn matching_requires_opposite_sides_of_same_pair() {
    let tiles = mosaic_tiles(&[word(1, "生产", "produce"), word(2, "发展", "develop")], 3);
    let matched = BTreeSet::new();
    let chinese = tiles
        .iter()
        .position(|tile| tile.pair == 0 && !tile.is_english)
        .unwrap();
    let english = tiles
        .iter()
        .position(|tile| tile.pair == 0 && tile.is_english)
        .unwrap();
    let wrong = tiles
        .iter()
        .position(|tile| tile.pair == 1 && tile.is_english)
        .unwrap();
    assert_eq!(
        choose_tile(&tiles, &matched, None, chinese),
        MosaicChoice::Selected(chinese)
    );
    assert_eq!(
        choose_tile(&tiles, &matched, Some(chinese), chinese),
        MosaicChoice::Deselected
    );
    assert_eq!(
        choose_tile(&tiles, &matched, Some(chinese), wrong),
        MosaicChoice::Incorrect
    );
    assert_eq!(
        choose_tile(&tiles, &matched, Some(chinese), english),
        MosaicChoice::Matched(0)
    );
}

#[test]
fn matched_tiles_cannot_be_selected_or_used_again() {
    let tiles = mosaic_tiles(&[word(1, "生产", "produce")], 3);
    let matched = BTreeSet::from([0]);
    assert_eq!(
        choose_tile(&tiles, &matched, None, 0),
        MosaicChoice::Unavailable
    );
    assert_eq!(
        choose_tile(&tiles, &matched, None, 10),
        MosaicChoice::Unavailable
    );
    assert_eq!(
        choose_tile(&tiles, &matched, Some(0), 1),
        MosaicChoice::Unavailable
    );
    assert!(mosaic_complete(&matched, &tiles));
    assert!(!mosaic_complete(&BTreeSet::new(), &tiles));
    assert!(!mosaic_complete(&BTreeSet::new(), &[]));
}

#[test]
fn unrelated_pair_ids_do_not_count_as_completed_mosaic() {
    let tiles = mosaic_tiles(&[word(1, "生产", "produce")], 3);
    assert!(!mosaic_complete(&BTreeSet::from([99]), &tiles));
}

#[test]
fn shuffled_mosaic_preserves_pair_identity_and_reproducibility() {
    let words = [
        word(1, "生产", "produce"),
        word(2, "发展", "develop"),
        word(3, "学习", "study"),
    ];
    let one = mosaic_tiles(&words, 101);
    assert_eq!(one, mosaic_tiles(&words, 101));
    assert_eq!(one.len(), 6);
    assert!(mosaic_complete(&BTreeSet::from([0, 1, 2]), &one));
    assert!(!mosaic_complete(&BTreeSet::from([0, 2]), &one));
}

#[test]
fn incomplete_or_malformed_tile_pairs_never_count_as_complete() {
    let tiles = vec![
        MosaicTile {
            pair: 0,
            is_english: false,
            label: "中".into(),
        },
        MosaicTile {
            pair: 0,
            is_english: false,
            label: "中".into(),
        },
    ];
    assert!(!mosaic_complete(&BTreeSet::from([0]), &tiles));
    let unmatched = vec![
        MosaicTile {
            pair: 0,
            is_english: false,
            label: "中".into(),
        },
        MosaicTile {
            pair: 1,
            is_english: true,
            label: "middle".into(),
        },
    ];
    assert!(!mosaic_complete(&BTreeSet::from([0]), &unmatched));
}
