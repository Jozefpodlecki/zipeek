use std::future::Future;

use futures::channel::oneshot;
use futures::future::{Either, select};
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::Window;

use super::{DbError, error::js_error};

pub async fn with_deadline<T, F>(
    window: &Window,
    milliseconds: i32,
    operation: &'static str,
    task: F,
) -> Result<T, DbError>
where
    F: Future<Output = Result<T, DbError>>,
{
    let (signal, timeout) = oneshot::channel::<()>();
    let mut signal = Some(signal);
    let callback = Closure::<dyn FnMut()>::new(move || {
        if let Some(signal) = signal.take() {
            let _ = signal.send(());
        }
    });
    let handle = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            milliseconds,
        )
        .map_err(|error| js_error("Scheduling database initialization deadline", error))?;
    futures::pin_mut!(task);
    futures::pin_mut!(timeout);
    let result = match select(task, timeout).await {
        Either::Left((result, _)) => result,
        Either::Right((_, _)) => Err(DbError::Operation {
            operation,
            message: "Database initialization timed out; retry the download".into(),
        }),
    };
    window.clear_timeout_with_handle(handle);
    result
}
