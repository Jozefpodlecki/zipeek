use super::rotation::{CrosswordRotation, HISTORY_KEY};
use crate::AppEnvironment;
use serde::{Deserialize, Serialize};
use zipeek_exercises::crossword::{Crossword, CrosswordWord, Placement};
use zipeek_exercises::practice_draft::HskBand;

pub(super) fn load_rotation(environment: &AppEnvironment) -> CrosswordRotation {
    match environment.local_storage.get_item(HISTORY_KEY) {
        Ok(Some(raw)) if raw.len() > 65_536 => {
            log::warn!("Crossword rotation history exceeds the permitted size");
            CrosswordRotation::default()
        }
        Ok(Some(raw)) => match serde_json::from_str::<CrosswordRotation>(&raw) {
            Ok(history) if history.valid() => history,
            _ => {
                log::warn!("Invalid crossword rotation history, starting a new local rotation");
                CrosswordRotation::default()
            }
        },
        Ok(None) => CrosswordRotation::default(),
        Err(error) => {
            log::warn!("Crossword rotation history unavailable: {error:?}");
            CrosswordRotation::default()
        }
    }
}

pub(super) fn record_rotation(environment: &AppEnvironment, session_id: &str, game: &Crossword) {
    if !valid_session_id(session_id) {
        return;
    }
    let mut history = load_rotation(environment);
    if !history.record(session_id, game) {
        return;
    }
    match serde_json::to_string(&history) {
        Ok(raw) => {
            if let Err(error) = environment.local_storage.set_item(HISTORY_KEY, &raw) {
                log::warn!("Crossword rotation history could not be saved: {error:?}");
            }
        }
        Err(error) => log::warn!("Crossword rotation serialization failed: {error}"),
    }
}

const STORAGE_VERSION: u8 = 5;
const KEY_PREFIX: &str = "zipeek.crosswords.v5.";

#[derive(Serialize, Deserialize)]
struct SavedCell {
    row: usize,
    col: usize,
    character: char,
}

#[derive(Serialize, Deserialize)]
struct SavedWord {
    answer: String,
    english: String,
    pinyin: String,
}

#[derive(Serialize, Deserialize)]
struct SavedBand {
    system: u8,
    level: u8,
}

#[derive(Serialize, Deserialize)]
struct SavedGame {
    version: u8,
    seed: u64,
    bands: Vec<SavedBand>,
    vocabulary: Vec<SavedWord>,
    cells: Vec<SavedCell>,
    mistakes: u32,
}

fn key(session_id: &str) -> String {
    format!("{KEY_PREFIX}{session_id}")
}

pub(super) fn save(
    environment: &AppEnvironment,
    session_id: &str,
    game: &Crossword,
    bands: &[HskBand],
) {
    let storage = &environment.local_storage;
    let snapshot = SavedGame {
        version: STORAGE_VERSION,
        seed: game.seed(),
        bands: bands
            .iter()
            .map(|band| SavedBand {
                system: band.system,
                level: band.level,
            })
            .collect(),
        vocabulary: game
            .vocabulary()
            .iter()
            .map(|word| SavedWord {
                answer: word.answer.clone(),
                english: word.english.clone(),
                pinyin: word.pinyin.clone(),
            })
            .collect(),
        cells: game
            .cells()
            .filter_map(|((row, col), _, character)| {
                character.map(|character| SavedCell {
                    row,
                    col,
                    character,
                })
            })
            .collect(),
        mistakes: game.mistakes,
    };
    if let Ok(raw) = serde_json::to_string(&snapshot)
        && let Err(error) = storage.set_item(&key(session_id), &raw)
    {
        log::warn!("Crossword progress could not be saved: {error:?}");
    }
}

pub(super) fn load(
    environment: &AppEnvironment,
    session_id: &str,
) -> Option<(Crossword, Vec<HskBand>)> {
    let raw = environment
        .local_storage
        .get_item(&key(session_id))
        .ok()??;
    let saved: SavedGame = serde_json::from_str(&raw).ok()?;
    if saved.version != STORAGE_VERSION
        || saved.vocabulary.len() > 128
        || saved.cells.len() > 225
        || saved.bands.is_empty()
        || saved.bands.len() > 13
    {
        return None;
    }
    let bands = saved
        .bands
        .into_iter()
        .map(|band| HskBand::new(band.system, band.level).ok())
        .collect::<Option<Vec<_>>>()?;
    let vocabulary = saved
        .vocabulary
        .into_iter()
        .map(|word| CrosswordWord {
            answer: word.answer,
            english: word.english,
            pinyin: word.pinyin,
        })
        .collect();
    let mut game = Crossword::generated(saved.seed, vocabulary);
    if game.clues().len() < 2 || saved.cells.len() > game.total() {
        return None;
    }
    for cell in saved.cells {
        let index = game
            .bank()
            .iter()
            .position(|character| *character == cell.character)?;
        if !matches!(
            game.place(cell.row, cell.col, index),
            Placement::Correct | Placement::Finished
        ) {
            return None;
        }
    }
    game.mistakes = saved.mistakes;
    game.solution_is_consistent().then_some((game, bands))
}

pub(super) fn seed_from_session_id(id: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in id.bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    hash
}

pub(super) fn session_id_from_query(query_id: Option<&str>) -> Option<String> {
    query_id
        .filter(|id| valid_session_id(id))
        .map(str::to_owned)
}

fn valid_session_id(value: &str) -> bool {
    value.len() == 36
        && value.chars().enumerate().all(|(index, ch)| {
            if [8, 13, 18, 23].contains(&index) {
                ch == '-'
            } else {
                ch.is_ascii_hexdigit()
            }
        })
}

pub(super) fn new_session_id() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        log::warn!("Secure randomness unavailable for crossword session ID");
        return String::new();
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}-{:02x}-{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

pub(super) fn update_url(environment: &AppEnvironment, session_id: &str) {
    if !valid_session_id(session_id) {
        return;
    }
    let hash = format!("/games/crosswords?session_id={session_id}");
    if let Err(error) = environment.window.location().set_hash(&hash) {
        log::warn!("Could not update crossword session URL: {error:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn different_session_ids_produce_stable_distinct_seeds() {
        let first = "550e8400-e29b-41d4-a716-446655440000";
        let second = "550e8400-e29b-41d4-a716-446655440001";
        assert_eq!(seed_from_session_id(first), seed_from_session_id(first));
        assert_ne!(seed_from_session_id(first), seed_from_session_id(second));
    }
    #[test]
    fn session_ids_are_uuid_shaped() {
        assert!(valid_session_id("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!valid_session_id("../something"));
    }
}
