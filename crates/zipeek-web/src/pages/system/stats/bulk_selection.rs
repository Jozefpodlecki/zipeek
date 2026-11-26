use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

#[derive(Clone)]
pub(super) struct BulkSelection<Q> {
    query: Rc<RefCell<Q>>,
    active: Rc<RefCell<bool>>,
    generation: Rc<RefCell<u64>>,
    pub loading: UseStateHandle<bool>,
    pub error: UseStateHandle<Option<String>>,
}

impl<Q: PartialEq> BulkSelection<Q> {
    pub fn begin(&self) -> u64 {
        let mut generation = self.generation.borrow_mut();
        *generation = generation.wrapping_add(1);
        let ticket = *generation;
        self.loading.set(true);
        self.error.set(None);
        ticket
    }

    pub fn current(&self, query: &Q, ticket: u64) -> bool {
        *self.active.borrow()
            && *self.query.borrow() == *query
            && *self.generation.borrow() == ticket
    }

    pub fn finish(&self, result: Result<Vec<String>, String>) -> Option<Vec<String>> {
        self.loading.set(false);
        match result {
            Ok(ids) => Some(ids),
            Err(message) => {
                self.error.set(Some(message));
                None
            }
        }
    }

    pub fn cancel(&self) {
        let mut generation = self.generation.borrow_mut();
        *generation = generation.wrapping_add(1);
        self.loading.set(false);
        self.error.set(None);
    }
}

#[hook]
pub(super) fn use_bulk_selection<Q: Clone + PartialEq + 'static>(query: Q) -> BulkSelection<Q> {
    let current = use_mut_ref(|| query.clone());
    *current.borrow_mut() = query.clone();
    let active = use_mut_ref(|| true);
    let generation = use_mut_ref(|| 0_u64);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    {
        let generation = generation.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with(query, move |_| {
            let mut value = generation.borrow_mut();
            *value = value.wrapping_add(1);
            loading.set(false);
            error.set(None);
        });
    }
    {
        let active = active.clone();
        use_effect_with((), move |_| move || *active.borrow_mut() = false);
    }
    BulkSelection {
        query: current,
        active,
        generation,
        loading,
        error,
    }
}
