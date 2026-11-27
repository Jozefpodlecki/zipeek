use super::*;
use crate::sentence_builder::PROMPTS;

fn choose(state: &mut BuilderState, order: &[usize]) {
    for &index in order {
        state.apply(BuilderAction::Choose(TileId::new(index)));
    }
}

fn order(state: &BuilderState) -> Vec<usize> {
    state.selected().iter().map(|tile| tile.index()).collect()
}

#[test]
fn consecutive_actions_use_the_latest_selection_and_reject_duplicate_or_invalid_tiles() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    choose(&mut state, &[1, 2, 2, usize::MAX, 3]);
    assert_eq!(order(&state), [1, 2, 3]);
    state.apply(BuilderAction::Check);
    assert_eq!(state.verdict(), None);
    state.apply(BuilderAction::Continue);
    assert!(!state.completed());
}

#[test]
fn grading_and_retry_options_cover_all_four_combinations() {
    for automatic in [false, true] {
        for retry in [false, true] {
            let mut state = BuilderState::new(&PROMPTS[0], automatic, retry);
            choose(&mut state, &[0, 1, 2, 3]);
            assert_eq!(state.verdict(), automatic.then_some(Verdict::Incorrect));
            if !automatic {
                state.apply(BuilderAction::Check);
            }
            assert_eq!(state.verdict(), Some(Verdict::Incorrect));
            assert_eq!(state.locked(), !retry);
            assert_eq!(state.can_continue(), !retry);
            state.apply(BuilderAction::Retry);
            assert_eq!(
                state.verdict(),
                if retry {
                    None
                } else {
                    Some(Verdict::Incorrect)
                }
            );
            assert_eq!(order(&state), [0, 1, 2, 3]);
        }
    }
}

#[test]
fn auto_retry_does_not_immediately_regrade_an_unchanged_wrong_order() {
    let mut state = BuilderState::new(&PROMPTS[0], true, true);
    choose(&mut state, &[0, 1, 2, 3]);
    state.apply(BuilderAction::Retry);
    state.apply(BuilderAction::Check);
    state.apply(BuilderAction::Move(TileId::new(0), MoveDirection::Left));
    assert_eq!(state.verdict(), None);
    state.apply(BuilderAction::Move(TileId::new(0), MoveDirection::Right));
    assert_eq!(state.verdict(), Some(Verdict::Incorrect));
    state.apply(BuilderAction::Undo);
    assert_eq!(state.verdict(), None);
    assert!(!state.complete_order());
}

#[test]
fn reviewed_alternative_orders_are_accepted() {
    for &accepted in PROMPTS[1].accepted_orders {
        let mut state = BuilderState::new(&PROMPTS[1], true, true);
        choose(&mut state, accepted);
        assert_eq!(state.verdict(), Some(Verdict::Correct));
        assert!(state.can_continue());
    }
}

#[test]
fn locked_answers_reject_every_edit_and_drag_action() {
    for correct in [false, true] {
        let mut state = BuilderState::new(&PROMPTS[0], true, false);
        choose(
            &mut state,
            if correct {
                PROMPTS[0].accepted_orders[0]
            } else {
                &[0, 1, 2, 3]
            },
        );
        let checked = state.clone();
        for action in [
            BuilderAction::Choose(TileId::new(0)),
            BuilderAction::Remove(TileId::new(0)),
            BuilderAction::Move(TileId::new(0), MoveDirection::Right),
            BuilderAction::Undo,
            BuilderAction::Clear,
            BuilderAction::Check,
            BuilderAction::Retry,
            BuilderAction::BeginDrag(DragSource::Selection(TileId::new(0))),
            BuilderAction::BeginDrag(DragSource::Bank(TileId::new(0))),
            BuilderAction::Drop(DropTarget::End),
        ] {
            state.apply(action);
            assert_eq!(state, checked);
        }
    }
}

#[test]
fn dragging_from_bank_inserts_at_the_target_and_cannot_reuse_a_tile() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    choose(&mut state, &[1, 3]);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(2))));
    state.apply(BuilderAction::Drop(DropTarget::Tile(TileId::new(3))));
    assert_eq!(order(&state), [1, 2, 3]);
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [1, 2, 3]);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(2))));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [1, 2, 3]);
}

#[test]
fn selected_drag_and_keyboard_reordering_keep_stable_tile_identity() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    choose(&mut state, &[0, 1, 2]);
    state.apply(BuilderAction::BeginDrag(DragSource::Selection(
        TileId::new(0),
    )));
    state.apply(BuilderAction::Drop(DropTarget::Tile(TileId::new(2))));
    assert_eq!(order(&state), [1, 2, 0]);
    state.apply(BuilderAction::Move(TileId::new(0), MoveDirection::Left));
    assert_eq!(order(&state), [1, 0, 2]);
    state.apply(BuilderAction::BeginDrag(DragSource::Selection(
        TileId::new(1),
    )));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [0, 2, 1]);
}

#[test]
fn edits_cancel_stale_drags_and_invalid_drop_targets_do_not_change_the_answer() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    choose(&mut state, &[0, 1]);
    state.apply(BuilderAction::BeginDrag(DragSource::Selection(
        TileId::new(0),
    )));
    state.apply(BuilderAction::Remove(TileId::new(1)));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [0]);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(2))));
    state.apply(BuilderAction::Drop(DropTarget::Tile(TileId::new(99))));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [0]);
}

#[test]
fn invalid_drag_start_clears_the_previous_source() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(1))));
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(99))));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert!(state.selected().is_empty());
}

#[test]
fn stale_drag_end_cannot_cancel_a_different_current_drag() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(1))));
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(2))));
    state.apply(BuilderAction::CancelDrag(DragSource::Bank(TileId::new(1))));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(order(&state), [2]);
}

#[test]
fn auto_check_locks_in_the_same_transition_that_completes_the_answer() {
    let mut state = BuilderState::new(&PROMPTS[0], true, true);
    choose(&mut state, &PROMPTS[0].accepted_orders[0][..3]);
    state.apply(BuilderAction::BeginDrag(DragSource::Bank(TileId::new(0))));
    state.apply(BuilderAction::Drop(DropTarget::End));
    assert_eq!(state.verdict(), Some(Verdict::Correct));
    let checked = state.clone();
    state.apply(BuilderAction::Drop(DropTarget::End));
    state.apply(BuilderAction::Clear);
    assert_eq!(state, checked);
}

#[test]
fn completion_is_a_single_terminal_transition() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    state.apply(BuilderAction::Continue);
    assert!(!state.completed());
    choose(&mut state, PROMPTS[0].accepted_orders[0]);
    state.apply(BuilderAction::Check);
    state.apply(BuilderAction::Continue);
    assert!(state.completed());
    assert!(!state.can_continue());
    let completed = state.clone();
    for action in [
        BuilderAction::Continue,
        BuilderAction::Clear,
        BuilderAction::Retry,
        BuilderAction::Undo,
    ] {
        state.apply(action);
        assert_eq!(state, completed);
    }
}

#[test]
fn attempt_evidence_tracks_checks_and_mistakes_without_counting_edits() {
    let mut state = BuilderState::new(&PROMPTS[0], false, true);
    choose(&mut state, &[0, 1, 2, 3]);
    assert_eq!((state.checks(), state.mistakes()), (0, 0));
    state.apply(BuilderAction::Check);
    assert_eq!((state.checks(), state.mistakes()), (1, 1));
    state.apply(BuilderAction::Retry);
    // Retry and attempted checks on an unchanged order do not count as submissions.
    assert_eq!((state.checks(), state.mistakes()), (1, 1));
    state.apply(BuilderAction::Clear);
    choose(&mut state, PROMPTS[0].accepted_orders[0]);
    state.apply(BuilderAction::Check);
    assert_eq!(state.verdict(), Some(Verdict::Correct));
    assert_eq!((state.checks(), state.mistakes()), (2, 1));
    state.apply(BuilderAction::Continue);
    assert_eq!((state.checks(), state.mistakes()), (2, 1));
}

#[test]
fn automatic_check_records_one_attempt_per_complete_order() {
    let mut state = BuilderState::new(&PROMPTS[0], true, true);
    choose(&mut state, &[0, 1, 2, 3]);
    assert_eq!((state.checks(), state.mistakes()), (1, 1));
    state.apply(BuilderAction::Retry);
    state.apply(BuilderAction::Clear);
    choose(&mut state, PROMPTS[0].accepted_orders[0]);
    assert_eq!(state.verdict(), Some(Verdict::Correct));
    assert_eq!((state.checks(), state.mistakes()), (2, 1));
}
