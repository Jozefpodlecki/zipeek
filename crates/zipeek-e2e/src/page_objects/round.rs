use crate::HarnessResult;
use crate::page_objects::WebDriverActions;
use serde_json::Value;
use thirtyfour::prelude::*;

fn parse_cards(raw: &str) -> HarnessResult<Vec<Vec<Value>>> {
    serde_json::from_str(raw)
        .map_err(|error| format!("Practice round fixture contains invalid JSON: {error}").into())
}

async fn cards(driver: &WebDriver) -> HarnessResult<Vec<Vec<Value>>> {
    let raw = driver
        .find_css("[data-practice-round]")
        .await?
        .attr("data-practice-round")
        .await?
        .ok_or("Practice fixture did not expose round data")?;
    parse_cards(&raw)
}

pub async fn mcq_correct_index(driver: &WebDriver) -> HarnessResult<usize> {
    let cards = cards(driver).await?;
    if cards.len() != 4 || cards.iter().any(|card| card.is_empty()) {
        return Err(format!(
            "MCQ fixture must contain a question and three complete options, got {} cards",
            cards.len()
        )
        .into());
    }
    let expected = &cards[0][0];
    cards[1..]
        .iter()
        .position(|card| card.first() == Some(expected))
        .ok_or("MCQ fixture has no matching correct option".into())
}

fn matching_indices(cards: &[Vec<Value>]) -> HarnessResult<Vec<usize>> {
    if cards.len() != 8 {
        return Err(format!("Matching fixture expected 8 cards, found {}", cards.len()).into());
    }
    let left_ids: Vec<_> = cards[..4].iter().filter_map(|card| card.first()).collect();
    if left_ids.len() != 4
        || left_ids
            .iter()
            .enumerate()
            .any(|(index, id)| left_ids[..index].contains(id))
    {
        return Err("Matching fixture contains missing or repeated left-hand word IDs".into());
    }
    let right_ids: Vec<_> = cards[4..].iter().filter_map(|card| card.first()).collect();
    if right_ids.len() != 4
        || right_ids
            .iter()
            .enumerate()
            .any(|(index, id)| right_ids[..index].contains(id))
    {
        return Err("Matching fixture contains missing or repeated right-hand word IDs".into());
    }
    cards[..4]
        .iter()
        .map(|left| {
            cards[4..]
                .iter()
                .position(|right| left.first().is_some() && left.first() == right.first())
                .ok_or_else(|| "Matching fixture contains a word without a partner".into())
        })
        .collect()
}

pub async fn matching_pairs(driver: &WebDriver) -> HarnessResult<Vec<usize>> {
    matching_indices(&cards(driver).await?)
}

pub async fn first_word_id(driver: &WebDriver) -> HarnessResult<String> {
    let cards = cards(driver).await?;
    let id = cards
        .first()
        .and_then(|card| card.first())
        .ok_or("Practice fixture has no first word ID")?;
    Ok(match id {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    })
}

pub async fn round_contains_word(driver: &WebDriver, word_id: &str) -> HarnessResult<bool> {
    let cards = cards(driver).await?;
    Ok(cards.iter().any(|card| {
        card.first().is_some_and(|value| match value {
            Value::String(id) => id == word_id,
            Value::Number(id) => id.to_string() == word_id,
            _ => false,
        })
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn fixture_parser_rejects_invalid_json_and_incorrect_shapes() {
        assert!(super::parse_cards("not json").is_err());
        assert!(super::parse_cards("{}").is_err());
        assert_eq!(super::parse_cards("[[1],[2]]").unwrap().len(), 2);
    }

    #[test]
    fn matching_fixture_rejects_duplicate_right_word_ids() {
        let cards = vec![
            vec![json!(1)],
            vec![json!(2)],
            vec![json!(3)],
            vec![json!(4)],
            vec![json!(1)],
            vec![json!(1)],
            vec![json!(3)],
            vec![json!(4)],
        ];
        let error = super::matching_indices(&cards).unwrap_err().to_string();
        assert!(error.contains("repeated right-hand word IDs"), "{error}");
    }

    #[test]
    fn matching_fixture_rejects_missing_partners_and_duplicate_left_ids() {
        let missing_partner = vec![
            vec![json!(1)],
            vec![json!(2)],
            vec![json!(3)],
            vec![json!(4)],
            vec![json!(1)],
            vec![json!(2)],
            vec![json!(3)],
            vec![json!(5)],
        ];
        assert!(super::matching_indices(&missing_partner).is_err());
        let duplicate_left = vec![
            vec![json!(1)],
            vec![json!(1)],
            vec![json!(3)],
            vec![json!(4)],
            vec![json!(1)],
            vec![json!(2)],
            vec![json!(3)],
            vec![json!(4)],
        ];
        assert!(super::matching_indices(&duplicate_left).is_err());
    }

    #[test]
    fn matching_fixture_reports_correct_shuffled_partner_indices() {
        let cards = vec![
            vec![json!(1)],
            vec![json!(2)],
            vec![json!(3)],
            vec![json!(4)],
            vec![json!(4)],
            vec![json!(2)],
            vec![json!(1)],
            vec![json!(3)],
        ];
        assert_eq!(super::matching_indices(&cards).unwrap(), vec![2, 1, 3, 0]);
    }
}
