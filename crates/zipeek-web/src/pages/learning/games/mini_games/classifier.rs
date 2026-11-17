use yew::prelude::*;
use zipeek_exercises::mini_games::ClassifierState;

#[derive(Properties, PartialEq)]
pub struct ClassifierRoundProps {
    pub game: ClassifierState,
    pub on_answer: Callback<usize>,
}

#[function_component(ClassifierRound)]
pub fn classifier_round(props: &ClassifierRoundProps) -> Html {
    let Some(question) = props.game.current() else {
        return Html::default();
    };
    html! {
        <section
            data-classifier-market=""
            class="space-y-6 rounded-2xl border border-line bg-raised p-6 text-center"
        >
            <p class="text-sm text-muted">{"Choose the measure word that fits the item."}</p>
            <p class="text-5xl font-bold" lang="zh-Hans">{format!("一__{}", question.noun)}</p>
            <p class="text-muted">{question.meaning}</p>
            <div class="grid grid-cols-3 gap-3">
                {for question.choices.iter().enumerate().map(|(index, choice)| {
                    let on_answer = props.on_answer.reform(move |_: MouseEvent| index);
                    html! {
                        <button
                            key={index}
                            type="button"
                            data-classifier-choice={index.to_string()}
                            disabled={props.game.last_wrong_choice == Some(index)}
                            onclick={on_answer}
                            class="min-h-20 rounded-xl border border-line bg-surface text-3xl font-bold hover:border-accent focus-visible:outline-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            {*choice}
                        </button>
                    }
                })}
            </div>
        </section>
    }
}
