use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SearchContext(UseStateHandle<String>);

impl SearchContext {
    pub fn new(state: UseStateHandle<String>) -> Self {
        Self(state)
    }

    pub fn get(&self) -> &String {
        &*self.0
    }

    pub fn set(&self, value: String) {
        self.0.set(value);
    }
}