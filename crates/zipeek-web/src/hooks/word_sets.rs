use yew::prelude::*;

use crate::components::providers::word_sets_provider::WordSetsContext;

#[hook]
pub(crate) fn use_word_sets() -> WordSetsContext {
    use_context::<WordSetsContext>().expect("WordSetsProvider must wrap Word Set consumers")
}
