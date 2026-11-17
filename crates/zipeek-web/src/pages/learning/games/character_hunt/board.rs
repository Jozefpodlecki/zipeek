use yew::prelude::*;
use zipeek_exercises::character_hunt::Difficulty;

use super::{state::HuntSessionContext, tile::HuntTile};

#[function_component(HuntBoard)]
pub(super) fn hunt_board() -> Html {
    let context =
        use_context::<HuntSessionContext>().expect("HuntBoard must be rendered within HuntSession");
    let session = &context.0;
    let Some(board) = session.board.as_ref() else {
        return Html::default();
    };
    let columns = match session.difficulty {
        Difficulty::Easy => "grid-cols-2",
        Difficulty::Medium => "grid-cols-3",
        Difficulty::Difficult => "grid-cols-4",
        Difficulty::Hardcore => "grid-cols-5",
    };

    let element_classes_1 = classes!(
        "grid",
        columns,
        "w-full",
        "max-w-[min(100%,58vh)]",
        "gap-2",
        "rounded-2xl",
        "border",
        "border-line",
        "bg-surface",
        "p-3",
        "sm:gap-3",
        "sm:p-4",
    );
    html! {
        <div
            class="flex w-full flex-col items-center gap-2"
        >
        <p
            role="status"
            aria-live="polite"
            data-hunt-progress=""
            class="text-sm text-muted"
        >
            {format!("{} of {} words found", board.found.len(), board.total_words())}
        </p>
        <div
            class={element_classes_1}
            role="group"
            aria-label={format!("Character Hunt {} by {} board, {} of {} words found", session.difficulty.size(), session.difficulty.size(), board.found.len(), board.total_words())}
            data-hunt-grid-size={session.difficulty.size().to_string()}
        >
            {for (0..board.tiles.len()).map(|index| html! {
                <HuntTile
                    key={index}
                    index={index}
                />
            })}
        </div>
        </div>
    }
}
