use crate::{components::ListenButton, reducers::matching::CardChoice};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct MatchCardProps {
    pub choice: CardChoice,
    pub label: AttrValue,
    pub traditional: Option<AttrValue>,
    pub pinyin: Option<AttrValue>,
    pub hanzi: bool,
    pub speak: Option<AttrValue>,
    pub selected: bool,
    pub wrong: bool,
    pub matched: bool,
    pub locked: bool,
    pub on_select: Callback<CardChoice>,
}

#[function_component(MatchCard)]
pub(crate) fn match_card(props: &MatchCardProps) -> Html {
    let choice = props.choice;
    let select = props.on_select.clone();
    html! {
        <div data-match-card="" class="relative flex min-w-0">
            <button type="button" data-match-side={match choice.side {crate::reducers::matching::Side::Left=>"left",crate::reducers::matching::Side::Right=>"right"}}
                data-match-index={choice.index.to_string()} data-match-feedback={if props.wrong {"wrong"} else if props.matched {"correct"} else {"none"}}
                data-match-selected={props.selected.to_string()} aria-pressed={props.selected.to_string()}
                disabled={props.matched || props.locked}
                class={classes!("relative","flex","min-h-22","flex-1","w-full","min-w-0","items-center","justify-center","rounded-xl","border-2","px-3","py-2","text-center","pr-14",
                    if props.wrong {"border-red-600 dark:border-red-400"} else if props.matched {"border-slate-200 dark:border-slate-700"} else if props.selected {"border-blue-600 dark:border-blue-400"} else {"border-slate-500 hover:border-blue-600 dark:border-slate-500 dark:hover:border-blue-400"},
                    if props.wrong {"bg-red-50 dark:bg-red-950"} else if props.matched {"bg-slate-100 dark:bg-slate-800"} else if props.selected {"bg-blue-50 dark:bg-blue-950"} else {"bg-white hover:bg-slate-50 dark:bg-slate-900 dark:hover:bg-slate-800"},
                    if props.wrong {"text-red-950 dark:text-red-100"} else if props.matched {"text-slate-600 dark:text-slate-300"} else {"text-ink"},
                    props.wrong.then_some("motion-safe:animate-match-shake"),props.matched.then_some("motion-safe:animate-match-dim"))}
                onclick={Callback::from(move|_|select.emit(choice))}>
                <span class="block min-w-0 max-w-full break-words leading-snug">
                    <span class={if props.hanzi {"block text-xl leading-tight font-medium md:text-2xl"} else {"block text-base md:text-lg"}}>{&props.label}</span>
                    {props.traditional.as_ref().map(|traditional|html!{<span data-traditional="" class="mt-0.5 block text-xs leading-4">{traditional}</span>}).unwrap_or_default()}
                    {props.pinyin.as_ref().map(|pinyin|html!{<span data-practice-pinyin="" class="mt-1 block text-sm leading-4">{pinyin}</span>}).unwrap_or_default()}
                </span>
                <span class={classes!("absolute","top-1","right-4","size-6","text-xl",props.matched.then_some("text-emerald-700 dark:text-emerald-400"))} aria-label={if props.wrong {Some("Incorrect")} else if props.matched {Some("Correct")} else {None}}>{if props.wrong {"✗"} else if props.matched {"✓"} else {""}}</span>
            </button>
            {props.speak.as_ref().map(|hanzi|html!{<div class="pointer-events-none absolute right-1 bottom-1 flex"><span class="pointer-events-auto"><ListenButton hanzi={hanzi.clone()} /></span></div>}).unwrap_or_default()}
        </div>
    }
}
