use crate::{
    components::Loader,
    contexts::database::DatabaseStatus,
    hooks::{use_database_commands, use_database_status},
};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct DatabaseBoundaryProps {
    pub children: Children,
}

#[function_component(DatabaseBoundary)]
pub fn database_boundary(props: &DatabaseBoundaryProps) -> Html {
    let status = use_database_status();
    let commands = use_database_commands();
    let on_retry = Callback::from(move |_| commands.retry());
    match status {
        DatabaseStatus::Loading => html! { <div
            data-database-state="loading"
        >
        <Loader />
        </div> },
        DatabaseStatus::Ready(metadata) => {
            html! { <div
                data-database-state="ready"
                data-database-rows={metadata.row_count.to_string()}
                data-database-engine={metadata.engine}
            >
                {for props.children.iter()}
            </div> }
        }
        DatabaseStatus::Failed(message) => {
            html! {
                <section
                    data-database-state="failed"
                    aria-labelledby="dictionary-error-title"
                    class="flex min-h-[65vh] items-center justify-center px-4 py-12"
                >
                    <div
                        class="w-full max-w-xl rounded-3xl border border-line bg-surface/95 px-6 py-9 text-center shadow-2xl sm:px-10 sm:py-12"
                    >
                        <div
                            aria-hidden="true"
                            class="mx-auto flex size-16 items-center justify-center rounded-2xl border border-line bg-raised text-accent"
                        >
                            <Icon
                                data={IconData::LUCIDE_DATABASE}
                                width="32px"
                                height="32px"
                            />
                        </div>
                        <p
                            class="mt-6 text-xs font-semibold uppercase tracking-[.14em] text-muted"
                        >
                            {"Dictionary connection failed"}
                        </p>
                        <h1
                            id="dictionary-error-title"
                            class="mt-3 text-3xl font-bold leading-tight tracking-tight text-ink sm:text-4xl"
                        >
                            {"We couldn't load your dictionary"}
                        </h1>
                        <p
                            role="alert"
                            class="mx-auto mt-4 max-w-md text-base leading-relaxed text-muted"
                        >
                            {"Zipeek couldn't initialize the vocabulary database. You can try connecting again."}
                        </p>
                        <p
                            data-database-error-message=""
                            class="mx-auto mt-3 max-w-md break-words text-sm text-muted"
                        >
                            {message}
                        </p>
                        <button
                            type="button"
                            class="mt-7 inline-flex min-h-12 items-center justify-center gap-2 rounded-xl bg-brand-action px-7 py-3 font-semibold text-white transition-colors duration-150 hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transition-none"
                            onclick={on_retry}
                            data-database-retry=""
                        >
                            <Icon
                                data={IconData::LUCIDE_REFRESH_CW}
                                width="18px"
                                height="18px"
                            />
                            {"Try again"}
                        </button>
                        <div
                            class="mt-8 rounded-xl border border-line bg-raised/50 px-5 py-4 text-left"
                        >
                            <div
                                class="flex items-center gap-2 text-sm font-semibold text-ink"
                            >
                                <Icon
                                    data={IconData::LUCIDE_INFO}
                                    width="17px"
                                    height="17px"
                                />
                                {"If the issue continues"}
                            </div>
                            <p
                                class="mt-2 text-sm leading-relaxed text-muted"
                            >
                                {"Reload the page. If the dictionary still fails to open, check the browser console for technical details."}
                            </p>
                        </div>
                    </div>
                </section>
            }
        }
    }
}
