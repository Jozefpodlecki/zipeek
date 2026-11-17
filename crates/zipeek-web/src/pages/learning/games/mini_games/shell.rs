use yew::prelude::*;
use yew_router::prelude::Link;

use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct GameShellProps {
    pub title: &'static str,
    pub source: String,
    pub step: usize,
    pub total: usize,
    pub mistakes: u32,
    pub notice: Option<&'static str>,
    pub children: Children,
}

#[function_component(GameShell)]
pub fn game_shell(props: &GameShellProps) -> Html {
    html! {
        <main data-mini-game={props.title} class="mx-auto flex w-full max-w-2xl flex-col gap-5 px-4 py-6">
            <header class="space-y-2">
                <Link<Route> to={Route::Games} classes="text-sm font-semibold text-accent hover:underline">
                    {"Back to games"}
                </Link<Route>>
                <h1 class="text-3xl font-bold">{props.title}</h1>
                <p data-game-source="" class="text-sm text-muted">{props.source.clone()}</p>
                <div class="flex justify-between text-sm text-muted">
                    <span data-game-progress="">{format!("{} / {}", props.step.min(props.total), props.total)}</span>
                    <span data-game-mistakes="">{format!("{} mistakes", props.mistakes)}</span>
                </div>
                <progress value={props.step.min(props.total).to_string()} max={props.total.to_string()} class="w-full" />
            </header>
            {for props.children.iter()}
            if props.step < props.total {
                if let Some(notice) = props.notice {
                    <p role="status" aria-live="polite" class="text-center text-sm text-muted">{notice}</p>
                }
            }
        </main>
    }
}
