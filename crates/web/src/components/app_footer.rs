use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(AppFooter)]
pub fn app_footer() -> Html {
    let year = js_sys::Date::new_0().get_full_year();
    html! {
        <footer class="flex w-full items-center gap-2 border-t border-line bg-surface px-5 py-2.5 text-xs leading-[1.6] text-muted backdrop-blur-[12px] md:px-[clamp(1.5rem,4vw,4.5rem)]" data-app-footer="">
            <p class="shrink-0 whitespace-nowrap">{format!("© {year} Jozef Podlecki")}</p>
            <span aria-hidden="true" class="shrink-0">{"·"}</span>
            <p class="min-w-0">
                {"Chinese dictionary data from "}
                <a class="underline underline-offset-[.2em] transition-colors duration-150 hover:text-ink motion-reduce:transition-none" href="https://cc-cedict.org/">{"CC-CEDICT"}</a>
                {" (cc-cedict.org), licensed "}
                <a class="underline underline-offset-[.2em] transition-colors duration-150 hover:text-ink motion-reduce:transition-none" href="https://creativecommons.org/licenses/by-sa/4.0/">{"CC BY-SA 4.0"}</a>
                {". "}
                <a class="underline underline-offset-[.2em] transition-colors duration-150 hover:text-ink motion-reduce:transition-none" href="ATTRIBUTION.md">{"Data attribution and notices"}</a>
            </p>
            <span aria-hidden="true" class="shrink-0">{"·"}</span>
            <a href="https://github.com/Jozefpodlecki/zipeek" aria-label="Zipeek repository on GitHub"
                target="_blank" rel="noopener noreferrer"
                class="grid size-6 shrink-0 place-items-center rounded text-muted transition-colors duration-150 hover:text-ink motion-reduce:transition-none">
                <span aria-hidden="true"><Icon data={IconData::SIMPLE_ICONS_GITHUB} width="18px" height="18px" /></span>
            </a>
        </footer>
    }
}
