use super::{SentencePrompt, grade};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileId(usize);

impl TileId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragSource {
    Bank(TileId),
    Selection(TileId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropTarget {
    Tile(TileId),
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Correct,
    Incorrect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Editing,
    Feedback(Verdict),
    Completed(Verdict),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuilderAction {
    Choose(TileId),
    Remove(TileId),
    Move(TileId, MoveDirection),
    Undo,
    Clear,
    Check,
    Retry,
    BeginDrag(DragSource),
    CancelDrag(DragSource),
    Drop(DropTarget),
    Continue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuilderState {
    prompt: &'static SentencePrompt,
    automatic_check: bool,
    retry_incorrect: bool,
    selected: Vec<TileId>,
    dragged: Option<DragSource>,
    phase: Phase,
    checks: u16,
    mistakes: u16,
}

impl BuilderState {
    pub fn new(
        prompt: &'static SentencePrompt,
        automatic_check: bool,
        retry_incorrect: bool,
    ) -> Self {
        Self {
            prompt,
            automatic_check,
            retry_incorrect,
            selected: Vec::new(),
            dragged: None,
            phase: Phase::Editing,
            checks: 0,
            mistakes: 0,
        }
    }

    pub fn selected(&self) -> &[TileId] {
        &self.selected
    }

    pub fn verdict(&self) -> Option<Verdict> {
        match self.phase {
            Phase::Editing => None,
            Phase::Feedback(verdict) | Phase::Completed(verdict) => Some(verdict),
        }
    }

    /// Number of actual submitted/auto-checked complete orders across retries.
    pub fn checks(&self) -> u16 {
        self.checks
    }

    /// Number of checked orders the evaluator rejected.
    pub fn mistakes(&self) -> u16 {
        self.mistakes
    }

    pub fn completed(&self) -> bool {
        matches!(self.phase, Phase::Completed(_))
    }

    pub fn locked(&self) -> bool {
        match self.phase {
            Phase::Editing => false,
            Phase::Feedback(Verdict::Incorrect) => !self.retry_incorrect,
            Phase::Feedback(Verdict::Correct) | Phase::Completed(_) => true,
        }
    }

    pub fn can_continue(&self) -> bool {
        matches!(self.phase, Phase::Feedback(Verdict::Correct))
            || matches!(self.phase, Phase::Feedback(Verdict::Incorrect)) && !self.retry_incorrect
    }

    pub fn complete_order(&self) -> bool {
        self.selected.len() == self.prompt.tiles.len()
    }

    fn can_choose(&self, tile: TileId) -> bool {
        tile.index() < self.prompt.tiles.len() && !self.selected.contains(&tile)
    }

    fn evaluate(&mut self) {
        let order: Vec<_> = self.selected.iter().map(|tile| tile.index()).collect();
        let correct = grade(self.prompt, &order);
        self.checks = self.checks.saturating_add(1);
        if !correct {
            self.mistakes = self.mistakes.saturating_add(1);
        }
        self.phase = Phase::Feedback(if correct {
            Verdict::Correct
        } else {
            Verdict::Incorrect
        });
        self.dragged = None;
    }

    fn edit(&mut self, selected: Vec<TileId>) {
        if selected != self.selected {
            self.selected = selected;
            self.phase = Phase::Editing;
            self.dragged = None;
            if self.automatic_check && self.complete_order() {
                self.evaluate();
            }
        }
    }

    pub fn apply(&mut self, action: BuilderAction) {
        if self.completed() {
            return;
        }
        match action {
            BuilderAction::Continue if self.can_continue() => {
                if let Phase::Feedback(verdict) = self.phase {
                    self.phase = Phase::Completed(verdict);
                    self.dragged = None;
                }
            }
            BuilderAction::CancelDrag(source) if self.dragged == Some(source) => {
                self.dragged = None;
            }
            BuilderAction::Retry
                if self.phase == Phase::Feedback(Verdict::Incorrect) && self.retry_incorrect =>
            {
                self.phase = Phase::Editing;
                self.dragged = None;
            }
            BuilderAction::Drop(target) => {
                let source = self.dragged.take();
                if self.locked() {
                    return;
                }
                let mut selected = self.selected.clone();
                let position = match target {
                    DropTarget::Tile(tile) => {
                        let Some(position) = selected.iter().position(|value| *value == tile)
                        else {
                            return;
                        };
                        position
                    }
                    DropTarget::End => selected.len(),
                };
                match source {
                    Some(DragSource::Bank(tile)) if self.can_choose(tile) => {
                        selected.insert(position, tile)
                    }
                    Some(DragSource::Selection(tile)) => {
                        let Some(from) = selected.iter().position(|value| *value == tile) else {
                            return;
                        };
                        selected.remove(from);
                        selected.insert(position.min(selected.len()), tile);
                    }
                    _ => return,
                }
                self.edit(selected);
            }
            _ if self.locked() => {}
            BuilderAction::Choose(tile) if self.can_choose(tile) => {
                let mut selected = self.selected.clone();
                selected.push(tile);
                self.edit(selected);
            }
            BuilderAction::Remove(tile) => {
                let selected = self
                    .selected
                    .iter()
                    .copied()
                    .filter(|value| *value != tile)
                    .collect();
                self.edit(selected);
            }
            BuilderAction::Move(tile, direction) => {
                let Some(from) = self.selected.iter().position(|value| *value == tile) else {
                    return;
                };
                let to = match direction {
                    MoveDirection::Left => from.checked_sub(1),
                    MoveDirection::Right => {
                        from.checked_add(1).filter(|to| *to < self.selected.len())
                    }
                };
                if let Some(to) = to {
                    let mut selected = self.selected.clone();
                    selected.swap(from, to);
                    self.edit(selected);
                }
            }
            BuilderAction::Undo => {
                let mut selected = self.selected.clone();
                selected.pop();
                self.edit(selected);
            }
            BuilderAction::Clear => self.edit(Vec::new()),
            BuilderAction::Check if !self.automatic_check && self.complete_order() => {
                self.evaluate()
            }
            BuilderAction::BeginDrag(source) => {
                let valid = match source {
                    DragSource::Bank(tile) => self.can_choose(tile),
                    DragSource::Selection(tile) => self.selected.contains(&tile),
                };
                self.dragged = valid.then_some(source);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
