use yew::prelude::*;

use crate::reducers::browse::BrowseState;

#[hook]
pub(crate) fn use_browse_state() -> UseReducerHandle<BrowseState> {
    use_context::<UseReducerHandle<BrowseState>>()
        .expect("BrowseProvider must wrap Browse components")
}
