use crate::services::exercise_types::EXERCISES;
use std::collections::HashMap;
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::planned_handoff::PlannedRoundTicket;

pub(crate) fn next_variant_for_ticket(
    query: &HashMap<String, String>,
    ticket: &PlannedRoundTicket,
) -> Option<ExerciseVariant> {
    let selected = query.get("types")?;
    EXERCISES.into_iter().find(|variant| {
        selected
            .split(',')
            .any(|value| value.trim() == variant.seed_token())
            && (if ticket.definition_id == "multiple-choice" {
                variant.is_mcq()
            } else if ticket.definition_id == "matching" {
                !variant.is_mcq()
            } else {
                false
            })
    })
}

const GENERATION: &str = "plan_generation";
const INSTANCE: &str = "plan_instance";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TicketQueryError {
    MissingField,
    InvalidGeneration,
    EmptyInstance,
}

pub(crate) fn write_ticket(query: &mut HashMap<String, String>, ticket: &PlannedRoundTicket) {
    query.insert(GENERATION.to_owned(), ticket.generation.to_string());
    query.insert(INSTANCE.to_owned(), ticket.instance_id.clone());
}

pub(crate) fn read_ticket(
    query: &HashMap<String, String>,
    definition_id: &'static str,
) -> Result<Option<PlannedRoundTicket>, TicketQueryError> {
    match (query.get(GENERATION), query.get(INSTANCE)) {
        (None, None) => Ok(None),
        (Some(generation), Some(instance_id)) => {
            let generation = generation
                .parse::<u64>()
                .map_err(|_| TicketQueryError::InvalidGeneration)?;
            if instance_id.is_empty()
                || instance_id.len() > 256
                || instance_id.trim() != instance_id
                || instance_id.chars().any(char::is_control)
            {
                return Err(TicketQueryError::EmptyInstance);
            }
            Ok(Some(PlannedRoundTicket {
                generation,
                instance_id: instance_id.clone(),
                definition_id,
            }))
        }
        _ => Err(TicketQueryError::MissingField),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_step_requires_selected_format_from_the_correct_family() {
        let ticket = PlannedRoundTicket {
            generation: 1,
            instance_id: "planned-1-matching".into(),
            definition_id: "matching",
        };
        assert_eq!(next_variant_for_ticket(&HashMap::new(), &ticket), None);
        let query = HashMap::from([("types".to_owned(), "hanzi-to-meaning".to_owned())]);
        assert_eq!(next_variant_for_ticket(&query, &ticket), None);
        let mcq = PlannedRoundTicket {
            definition_id: "multiple-choice",
            ..ticket
        };
        assert_eq!(
            next_variant_for_ticket(&query, &mcq),
            EXERCISES
                .into_iter()
                .find(|variant| variant.seed_token() == "hanzi-to-meaning")
        );
    }

    #[test]
    fn preserves_ordinary_practice_query_without_planned_fields() {
        let mut query = HashMap::from([("level".into(), "3".into())]);
        assert_eq!(read_ticket(&query, "matching"), Ok(None));
        let ticket = PlannedRoundTicket {
            generation: 9,
            instance_id: "planned-0-matching".into(),
            definition_id: "matching",
        };
        write_ticket(&mut query, &ticket);
        assert_eq!(read_ticket(&query, "matching"), Ok(Some(ticket)));
        assert_eq!(query.get("level").map(String::as_str), Some("3"));
    }

    #[test]
    fn partial_or_invalid_tickets_fail_closed() {
        let mut query = HashMap::new();
        query.insert(GENERATION.into(), "2".into());
        assert_eq!(
            read_ticket(&query, "matching"),
            Err(TicketQueryError::MissingField)
        );
        query.insert(INSTANCE.into(), "planned-0-matching".into());
        query.insert(GENERATION.into(), "NaN".into());
        assert_eq!(
            read_ticket(&query, "matching"),
            Err(TicketQueryError::InvalidGeneration)
        );
        query.insert(GENERATION.into(), "2".into());
        query.insert(INSTANCE.into(), "\n".into());
        assert_eq!(
            read_ticket(&query, "matching"),
            Err(TicketQueryError::EmptyInstance)
        );
        query.insert(INSTANCE.into(), format!("{}x", "a".repeat(256)));
        assert_eq!(
            read_ticket(&query, "matching"),
            Err(TicketQueryError::EmptyInstance)
        );
        query.insert(INSTANCE.into(), " planned ".into());
        assert_eq!(
            read_ticket(&query, "matching"),
            Err(TicketQueryError::EmptyInstance)
        );
    }
}
