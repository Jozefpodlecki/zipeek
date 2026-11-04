use crate::{
    components::{Loader, RoutePlaceholder},
    contexts::database::DatabaseStatus,
    hooks::{use_database_commands, use_database_status},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DatabaseBoundaryProps {
    pub children: Children,
}

#[function_component(DatabaseBoundary)]
pub fn database_boundary(props: &DatabaseBoundaryProps) -> Html {
    let status = use_database_status();
    let commands = use_database_commands();
    let (Some(status), Some(commands)) = (status, commands) else {
        log::error!("DatabaseBoundary requires DatabaseProvider");
        return html! { <RoutePlaceholder title="Dictionary unavailable" message="The dictionary could not be loaded. Please try again." /> };
    };
    match status {
        DatabaseStatus::Loading => html! { <div data-database-state="loading"><Loader /></div> },
        DatabaseStatus::Ready(metadata) => {
            html! { <div data-database-state="ready" data-database-rows={metadata.row_count.to_string()} data-database-engine={metadata.engine}>{for props.children.iter()}</div> }
        }
        DatabaseStatus::Failed(_) => {
            html! { <div data-database-state="failed"><RoutePlaceholder title="Dictionary unavailable" message="The dictionary could not be loaded. Please try again."><button type="button" class="rounded px-3 py-2 text-ink transition-colors duration-150 hover:bg-raised motion-reduce:transition-none" onclick={Callback::from(move |_| commands.retry())} data-database-retry="">{"Retry"}</button></RoutePlaceholder></div> }
        }
    }
}
