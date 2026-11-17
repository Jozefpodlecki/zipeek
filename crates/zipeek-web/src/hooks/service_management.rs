use yew::prelude::*;

use crate::contexts::service_management::{ServiceContext, ServiceHandle};

#[hook]
pub fn use_ai_service() -> ServiceHandle {
    use_context::<ServiceContext>()
        .expect("ServiceProvider must wrap the application")
        .ai
}

#[hook]
pub fn use_image_service() -> ServiceHandle {
    use_context::<ServiceContext>()
        .expect("ServiceProvider must wrap the application")
        .image
}
