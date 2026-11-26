use super::track_navigation::{CourseTrack, CourseTrackNavigation};
use crate::{
    components::providers::word_sets_provider::WordSetsCommand,
    contexts::database::DatabaseStatus,
    hooks::{use_database_commands, use_database_status, use_dictionary, use_word_sets},
    services::word_sets::{COURSE_WORD_SET_SIZE, CourseWordSetSource},
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};
use yew::prelude::*;
use zipeek_core::Word;
use zipeek_pedagogy::curriculum::VOCABULARY_REGIONS as COURSES;

mod data;
mod loading;
mod mosaic;
mod region_details;
mod state;
mod vocabulary_geometry;
mod word_explorer;

use super::workspace::CourseWorkspace;
use mosaic::VocabularyMosaic;
use region_details::RegionDetails;
use state::{RegionLoad, VocabularyAction, VocabularyState};
use word_explorer::WordExplorer;

const WORD_SET_SIZE: usize = COURSE_WORD_SET_SIZE;
const CACHE_LIMIT: usize = 4;
type WordCache = VecDeque<(usize, Rc<Vec<Word>>)>;

fn word_set_name(title: &str, index: usize, total: usize) -> String {
    if total == 1 {
        title.to_owned()
    } else {
        format!("{title} ({}/{total})", index + 1)
    }
}

fn cache_insert(cache: &mut WordCache, index: usize, words: Rc<Vec<Word>>) {
    cache.retain(|(key, _)| *key != index);
    cache.push_back((index, words));
    while cache.len() > CACHE_LIMIT {
        cache.pop_front();
    }
}

#[function_component(VocabularyCoursePage)]
pub fn vocabulary_course_page() -> Html {
    let state = use_reducer_eq(VocabularyState::default);
    let dictionary = use_dictionary();
    let database_status = use_database_status();
    let database_commands = use_database_commands();
    let sets = use_word_sets();
    let cache = use_mut_ref(WordCache::new);

    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        let cache = cache.clone();
        let database_status = database_status.clone();
        use_effect_with(
            (state.selected, state.request, database_status),
            move |(selected, request, status)| {
                let active = Rc::new(Cell::new(true));
                let pending = active.clone();
                let index = *selected;
                let request_id = *request;
                match status {
                    DatabaseStatus::Loading => {
                        cache.borrow_mut().clear();
                        state.dispatch(VocabularyAction::DictionaryWaiting {
                            selected: index,
                            request: request_id,
                        });
                    }
                    DatabaseStatus::Failed(message) => {
                        cache.borrow_mut().clear();
                        state.dispatch(VocabularyAction::DictionaryFailed {
                            selected: index,
                            request: request_id,
                            message: message.clone(),
                        });
                    }
                    DatabaseStatus::Ready(_) => {
                        let cached = cache
                            .borrow()
                            .iter()
                            .find(|(key, _)| *key == index)
                            .map(|(_, words)| words.clone());
                        if let Some(words) = cached {
                            state.dispatch(VocabularyAction::Loaded {
                                selected: index,
                                request: request_id,
                                result: Ok(words),
                            });
                        } else {
                            let dictionary = dictionary.clone();
                            let cache = cache.clone();
                            let state = state.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let result = async {
                                    let course = COURSES.get(index).ok_or_else(|| {
                                        "Vocabulary collection unavailable".to_owned()
                                    })?;
                                    let database =
                                        dictionary.database().map_err(|error| error.to_string())?;
                                    let matches = database
                                        .words_by_simplified(course.hanzi)
                                        .await
                                        .map_err(|error| error.to_string())?;
                                    data::resolve_course_words(course, matches).map(Rc::new)
                                }
                                .await;
                                if pending.get() {
                                    if let Ok(words) = &result {
                                        cache_insert(&mut cache.borrow_mut(), index, words.clone());
                                    }
                                    state.dispatch(VocabularyAction::Loaded {
                                        selected: index,
                                        request: request_id,
                                        result,
                                    });
                                }
                            });
                        }
                    }
                }
                move || active.set(false)
            },
        );
    }

    let on_retry = {
        let state = state.clone();
        let database_commands = database_commands.clone();
        let database_status = database_status.clone();
        Callback::from(move |_: MouseEvent| {
            if matches!(database_status, DatabaseStatus::Failed(_)) {
                database_commands.retry();
            } else {
                state.dispatch(VocabularyAction::Retry);
            }
        })
    };
    let on_select = {
        let state = state.clone();
        Callback::from(move |index: usize| state.dispatch(VocabularyAction::Select(index)))
    };
    let on_previous = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(VocabularyAction::PreviousPage))
    };
    let on_next = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(VocabularyAction::NextPage))
    };
    let on_acquire = {
        let execute = sets.execute.clone();
        let state = state.clone();
        Callback::from(move |index: usize| {
            if let (Some(course), RegionLoad::Ready(words)) =
                (COURSES.get(state.selected), &state.load)
            {
                let count = words.len().div_ceil(WORD_SET_SIZE);
                if let Some(chunk) = words.chunks(WORD_SET_SIZE).nth(index) {
                    execute.emit(WordSetsCommand::CreateFromCourse {
                        name: word_set_name(course.title, index, count),
                        word_ids: chunk.iter().map(|word| word.id.as_i32()).collect(),
                        source: CourseWordSetSource {
                            region_id: course.id,
                            set_index: (index + 1) as u16,
                        },
                    });
                }
            }
        })
    };
    html! {
        <section data-component="vocabulary-courses" class="mx-auto flex w-full max-w-[1500px] flex-col gap-3 py-1 lg:h-full lg:min-h-0 lg:overflow-hidden">
            <CourseTrackNavigation active={CourseTrack::Vocabulary} />
            <CourseWorkspace
                mosaic={html! { <VocabularyMosaic selected={state.selected} on_select={on_select} /> }}
                explorer={html! { <WordExplorer load={state.load.clone()} page={state.page} on_previous={on_previous} on_next={on_next} /> }}
                details={html! { <RegionDetails selected={state.selected} load={state.load.clone()} on_retry={on_retry} on_acquire={on_acquire} /> }}
            />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_word_sets_have_stable_distinct_names() {
        assert_eq!(word_set_name("Family", 0, 1), "Family");
        assert_eq!(word_set_name("Family", 0, 3), "Family (1/3)");
        assert_eq!(word_set_name("Family", 2, 3), "Family (3/3)");
        assert_eq!(WORD_SET_SIZE, 200);
        for region in &COURSES {
            let total = region.readings.len().div_ceil(WORD_SET_SIZE);
            let names: std::collections::BTreeSet<_> = (0..total)
                .map(|index| word_set_name(region.title, index, total))
                .collect();
            assert_eq!(names.len(), total);
            assert!(names.iter().all(|name| name.chars().count() <= 64));
        }
    }

    #[test]
    fn all_one_hundred_regions_have_acquirable_collections() {
        assert_eq!(COURSES.len(), 100);
        let titles: std::collections::BTreeSet<_> =
            COURSES.iter().map(|course| course.title).collect();
        assert_eq!(titles.len(), 100);
    }

    #[test]
    fn authored_groups_have_unique_nonempty_hanzi() {
        for course in &COURSES {
            let unique: std::collections::BTreeSet<_> = course.hanzi.iter().collect();
            assert!(course.hanzi.len() >= 10);
            assert_eq!(unique.len(), course.hanzi.len());
        }
    }

    #[test]
    fn region_cache_is_bounded_and_replaces_existing_entries() {
        let mut cache = WordCache::new();
        for index in 0..6 {
            cache_insert(&mut cache, index, Rc::new(Vec::new()));
        }
        assert_eq!(cache.len(), CACHE_LIMIT);
        assert_eq!(cache.front().map(|(index, _)| *index), Some(2));
        cache_insert(&mut cache, 3, Rc::new(Vec::new()));
        assert_eq!(cache.len(), CACHE_LIMIT);
        assert_eq!(cache.back().map(|(index, _)| *index), Some(3));
    }
}
