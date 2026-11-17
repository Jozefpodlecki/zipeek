use yew::prelude::*;

use super::state::HuntSessionContext;

#[function_component(HuntExit)]
pub(super) fn hunt_exit() -> Html {
    let context =
        use_context::<HuntSessionContext>().expect("HuntExit must be rendered within HuntSession");
    let session = &context.0;
    let finished =
        session.is_last_round() && session.board.as_ref().is_some_and(|board| board.complete());
    let confirming = use_state(|| false);
    let show = {
        let confirming = confirming.clone();
        Callback::from(move |_| confirming.set(true))
    };
    let cancel = {
        let confirming = confirming.clone();
        Callback::from(move |_| confirming.set(false))
    };

    if finished {
        return Html::default();
    }

    let exit_confirmation_content = {
        if *confirming {
            html! {
                <div
                    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
                >
                    <div
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="hunt-exit-title"
                        data-hunt-exit-dialog=""
                        class="w-full max-w-md space-y-3 rounded-2xl border border-line bg-surface p-6 shadow-xl"
                    >
                        <h2
                            id="hunt-exit-title"
                            class="font-semibold"
                        >
                            {"Leave this practice session?"}
                        </h2>
                        <p
                            class="text-sm text-muted"
                        >
                            {"Current session progress will be lost."}
                        </p>
                        <div
                            class="flex flex-wrap gap-3"
                        >
                        <button
                            type="button"
                            onclick={cancel}
                            class="min-h-11 rounded-xl bg-brand-action px-4 py-2 text-white"
                        >
                            {"Keep playing"}
                        </button>
                        <a
                            href="#/practice"
                            data-hunt-confirm-exit=""
                            class="inline-flex min-h-11 items-center rounded-xl border border-line px-4 py-2 text-ink no-underline"
                        >
                            {"Leave practice"}
                        </a>
                        </div>
                    </div>
                </div>
            }
        } else {
            Html::default()
        }
    };
    html! {
        <>
            <button
                type="button"
                data-hunt-exit=""
                onclick={show}
                class="min-h-11 rounded-xl border border-line bg-surface px-4 py-2 hover:border-accent"
            >
                {"Exit practice"}
            </button>
            {exit_confirmation_content}
        </>
    }
}
