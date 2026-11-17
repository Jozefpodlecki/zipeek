use std::collections::BTreeSet;
use yew::prelude::*;
use zipeek_exercises::image_questions::ImageQuestion;

#[derive(Properties, PartialEq)]
pub(super) struct Props {
    pub question: ImageQuestion,
    pub selected: BTreeSet<i32>,
    pub checked: bool,
    pub on_toggle: Callback<i32>,
}

#[function_component(AnswerChoices)]
pub(super) fn answer_choices(props: &Props) -> Html {
    html! {
        <div
            data-image-answer-choices="true"
            role="group"
            aria-label="Select matching Chinese words"
            class="grid gap-2 sm:grid-cols-3"
        >
            {for props.question.options.iter().map(|word| {
                let chosen = props.selected.contains(&word.id);
                let correct = props.question.correct_ids.contains(&word.id);
                let on_toggle = props.on_toggle.clone(); let id = word.id;
                let on_choose_answer = Callback::from(move |_| on_toggle.emit(id));
                let element_classes_1 = classes!(
                                "min-h-20",
                                "rounded-xl",
                                "border-2",
                                "p-4",
                                "text-2xl",
                                "font-semibold",
                                "disabled:cursor-default",
                                if props.checked && correct {"border-green-600"} else if chosen {"border-brand-action bg-accent-soft"} else {"border-line"},
                            );
                html! {
                    <button
                        key={word.id}
                        data-image-answer-option={word.id.to_string()}
                        type="button"
                        disabled={props.checked}
                        aria-pressed={chosen.to_string()}
                        class={element_classes_1}
                        onclick={on_choose_answer}
                    >
                        {&word.hanzi}
                    </button>
                }
            })}
        </div>
    }
}
