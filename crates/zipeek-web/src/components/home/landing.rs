use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::onboarding::OnboardingForm;

#[derive(Properties, PartialEq)]
pub(crate) struct LandingProps {
    pub on_enter: Callback<MouseEvent>,
}

#[function_component(LandingPage)]
pub(crate) fn landing_page(props: &LandingProps) -> Html {
    let configuring = use_state(|| false);
    let on_begin = {
        let configuring = configuring.clone();
        Callback::from(move |_: MouseEvent| configuring.set(true))
    };
    let on_back = {
        let configuring = configuring.clone();
        Callback::from(move |_: MouseEvent| configuring.set(false))
    };
    if *configuring {
        return html! { <OnboardingForm on_enter={props.on_enter.clone()} on_back={on_back} /> };
    }
    html! {
        <div
            data-component="welcome-landing"
            class="relative isolate flex min-h-dvh flex-col overflow-hidden bg-canvas text-ink"
        >
            <div
                aria-hidden="true"
                class="pointer-events-none absolute inset-0 opacity-[.82] [background:radial-gradient(ellipse_45%_50%_at_78%_41%,var(--accent-soft),transparent_85%),radial-gradient(ellipse_35%_40%_at_12%_95%,var(--accent-soft),transparent_100%)]"
            />
            <header
                class="relative z-10 mx-auto flex w-full max-w-7xl items-center px-6 py-6 sm:px-10"
            >
                <div
                    class="flex items-center gap-3"
                >
                    <img
                        src="public/favicon-32x32.png"
                        alt=""
                        class="size-9"
                    />
                    <strong
                        class="text-xl tracking-tight"
                    >
                        {"Zipeek"}
                    </strong>
                </div>
            </header>
            <main
                class="relative z-10 mx-auto grid w-full max-w-7xl flex-1 items-center gap-12 px-6 py-12 sm:px-10 lg:grid-cols-[1.1fr_.9fr] lg:gap-16"
            >
                <section
                    class="flex max-w-2xl flex-col items-start gap-7"
                >
                    <span
                        class="rounded-full border border-[var(--border)] bg-[var(--accent-soft)] text-[var(--accent)] px-4 py-2 text-xs font-bold tracking-[.15em] uppercase"
                    >
                        {"A quieter way to learn Mandarin"}
                    </span>
                    <h1
                        class="text-5xl leading-[1.1] font-extrabold tracking-tight sm:text-6xl xl:text-7xl"
                    >
                        {"Make Chinese "}<span
                            class="text-[var(--accent)]"
                        >
                        {"click."}
                        </span>
                        </h1>
                    <p
                        class="max-w-xl text-lg leading-relaxed text-muted sm:text-xl"
                    >
                        {"Practice vocabulary, explore readings and meanings, and build your own word sets — all in your browser."}
                    </p>
                    <button
                        type="button"
                        data-welcome-enter=""
                        onclick={on_begin}
                        class="inline-flex bg-brand-action text-white min-h-12 items-center justify-center gap-3 rounded-xl px-8 py-4 text-base font-bold shadow-xl transition-transform duration-200 hover:-translate-y-1 focus-visible:outline-offset-4 motion-reduce:transform-none motion-reduce:transition-none"
                    >
                        <Icon
                            data={IconData::LUCIDE_PLAY}
                            width="18px"
                            height="18px"
                        />
                        {"Enter Zipeek"}
                    </button>
                    <p
                        class="text-sm text-muted"
                    >
                        {"No account required. Your learning stays in your browser."}
                    </p>
                </section>
                <LandingPreview />
            </main>
        </div>
    }
}

#[function_component(LandingPreview)]
fn landing_preview() -> Html {
    html! {
        <section
            data-landing-preview=""
            aria-label="Preview of Zipeek learning features"
            class="relative backdrop-blur-[20px] mx-auto w-full max-w-xl overflow-hidden rounded-[2rem] border border-line bg-surface p-4 shadow-2xl sm:p-5"
        >
            <div
                class="flex items-center justify-between gap-3 px-1 pb-4"
            >
                <span
                    class="text-sm font-semibold tracking-wide text-ink"
                >
                    {"Inside Zipeek"}
                </span>
                <span
                    class="rounded-full border border-line px-3 py-1 text-xs text-muted"
                >
                    {"Product preview"}
                </span>
            </div>
            <div
                class="relative [background:radial-gradient(ellipse_at_50%_38%,var(--accent-soft),var(--canvas)_85%)] flex aspect-video flex-col items-center justify-center gap-3 overflow-hidden rounded-2xl border border-line p-5 text-center sm:gap-4"
            >
                <span
                    class="text-xs font-semibold tracking-[.16em] text-muted uppercase"
                >
                    {"Vocabulary practice"}
                </span>
                <span
                    lang="zh-Hans"
                    class="font-[family-name:var(--hanzi-font)] text-[var(--accent)] text-6xl leading-tight font-semibold sm:text-7xl"
                >
                    {"学习"}
                </span>
                <span
                    class="text-base font-medium text-ink"
                >
                    {"xué xí"}
                </span>
                <span
                    class="rounded-lg border border-line bg-surface px-5 py-2 text-sm text-muted"
                >
                    {"to study · to learn"}
                </span>
            </div>
            <div
                class="grid grid-cols-3 gap-2 pt-4 sm:gap-3"
            >
                <div
                    class="flex min-w-0 flex-col items-center gap-2 rounded-xl border border-line bg-raised px-2 py-3 text-center"
                >
                    <Icon
                        data={IconData::LUCIDE_GRADUATION_CAP}
                        width="22px"
                        height="22px"
                    />
                    <span
                        class="text-xs font-semibold sm:text-sm"
                    >
                        {"Practice"}
                    </span>
                </div>
                <div
                    class="flex min-w-0 flex-col items-center gap-2 rounded-xl border border-line bg-raised px-2 py-3 text-center"
                >
                    <Icon
                        data={IconData::LUCIDE_BOOK_OPEN}
                        width="22px"
                        height="22px"
                    />
                    <span
                        class="text-xs font-semibold sm:text-sm"
                    >
                        {"Browse"}
                    </span>
                </div>
                <div
                    class="flex min-w-0 flex-col items-center gap-2 rounded-xl border border-line bg-raised px-2 py-3 text-center"
                >
                    <Icon
                        data={IconData::LUCIDE_BOOKMARK}
                        width="22px"
                        height="22px"
                    />
                    <span
                        class="text-xs font-semibold sm:text-sm"
                    >
                        {"Word sets"}
                    </span>
                </div>
            </div>
        </section>
    }
}
