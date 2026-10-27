use yew::prelude::*;

#[function_component(AppFooter)]
pub fn app_footer() -> Html {
    let year = js_sys::Date::new_0().get_full_year();
    html! {
        <footer class="app-footer" data-app-footer="">
            <p>{format!("© {year} Jozef Podlecki")}</p>
            <p>
                {"Chinese dictionary data from "}
                <a href="https://cc-cedict.org/">{"CC-CEDICT"}</a>
                {", licensed "}
                <a href="https://creativecommons.org/licenses/by-sa/4.0/">{"CC BY-SA 4.0"}</a>
                {". "}
                <a href="ATTRIBUTION.md">{"Data attribution and notices"}</a>
            </p>
        </footer>
    }
}
