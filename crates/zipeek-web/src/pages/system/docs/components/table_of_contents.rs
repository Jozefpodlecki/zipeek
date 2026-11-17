use yew::prelude::*;
use zipeek_docs::DocumentHeading;

use super::super::navigation::scroll_to_heading;

#[derive(Properties, PartialEq)]
pub(in crate::pages::system::docs) struct TableOfContentsProps {
    pub toc: Vec<DocumentHeading>,
}

#[function_component(DocsTableOfContents)]
pub(in crate::pages::system::docs) fn docs_table_of_contents(props: &TableOfContentsProps) -> Html {
    let toc = &props.toc;
    html! {
                <aside aria-label="On this page" class={classes!(if toc.is_empty() {"hidden"} else {"hidden xl:block"}, "xl:sticky", "xl:top-[4.5rem]", "xl:h-[calc(100dvh-4.5rem)]", "xl:overflow-auto", "border-l", "border-line", "p-5")}>
                    <h2 class="text-sm font-bold mb-4">{"On this page"}</h2>
                    {for toc.iter().map(|heading| {
                        let id = heading.id.clone();
                        let onclick = Callback::from(move |_| {
                            scroll_to_heading(&id);
                        });
                        html! { <button type="button" key={heading.id.clone()} class={classes!("block", "w-full", "rounded-md", "text-left", "text-sm", "py-1", "hover:text-accent", if heading.level > 1 {"pl-3"} else {"font-semibold"})} {onclick}>{&heading.title}</button> }
                    })}
                </aside>
    }
}
