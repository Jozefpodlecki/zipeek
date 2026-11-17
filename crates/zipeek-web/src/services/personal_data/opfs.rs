use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::Navigator;

use super::error::PersonalDataError;

const SNAPSHOT_FILE: &str = "zipeek-learning-snapshot-v1.json";

fn describe(error: &JsValue) -> String {
    let name = Reflect::get(error, &JsValue::from_str("name"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default();
    let message = Reflect::get(error, &JsValue::from_str("message"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| format!("{error:?}"));
    format!("{name}: {message}")
}

fn property(object: &JsValue, name: &str) -> Result<JsValue, PersonalDataError> {
    Reflect::get(object, &JsValue::from_str(name)).map_err(|error| {
        PersonalDataError::Opfs(format!("Cannot read {name}: {}", describe(&error)))
    })
}

fn call(object: &JsValue, method: &str, args: &[JsValue]) -> Result<JsValue, PersonalDataError> {
    let function: Function = property(object, method)?
        .dyn_into()
        .map_err(|_| PersonalDataError::Opfs(format!("Browser does not support {method}")))?;
    let arguments = Array::new();
    for argument in args {
        arguments.push(argument);
    }
    function
        .apply(object, &arguments)
        .map_err(|error| PersonalDataError::Opfs(format!("{method} failed: {}", describe(&error))))
}

async fn invoke(
    object: &JsValue,
    method: &str,
    args: &[JsValue],
) -> Result<JsValue, PersonalDataError> {
    let value = call(object, method, args)?;
    JsFuture::from(Promise::resolve(&value))
        .await
        .map_err(|error| PersonalDataError::Opfs(format!("{method} failed: {}", describe(&error))))
}

fn storage(navigator: &Navigator) -> Result<JsValue, PersonalDataError> {
    let value = property(navigator.as_ref(), "storage")?;
    if value.is_null() || value.is_undefined() {
        return Err(PersonalDataError::Opfs(
            "Browser private storage is unavailable".into(),
        ));
    }
    Ok(value)
}

async fn directory(navigator: &Navigator) -> Result<JsValue, PersonalDataError> {
    invoke(&storage(navigator)?, "getDirectory", &[]).await
}

fn create_option() -> Result<JsValue, PersonalDataError> {
    let object = Object::new();
    Reflect::set(&object, &JsValue::from_str("create"), &JsValue::TRUE)
        .map_err(PersonalDataError::from)?;
    Ok(object.into())
}

async fn file_handle(
    root: &JsValue,
    name: &str,
    create: bool,
) -> Result<JsValue, PersonalDataError> {
    let args = if create {
        vec![JsValue::from_str(name), create_option()?]
    } else {
        vec![JsValue::from_str(name)]
    };
    invoke(root, "getFileHandle", &args).await
}

async fn read_file(root: &JsValue, name: &str) -> Result<String, PersonalDataError> {
    let handle = file_handle(root, name, false).await?;
    let file = invoke(&handle, "getFile", &[]).await?;
    invoke(&file, "text", &[])
        .await?
        .as_string()
        .ok_or_else(|| PersonalDataError::Opfs("File text was not a string".into()))
}

async fn write_file(root: &JsValue, name: &str, contents: &str) -> Result<(), PersonalDataError> {
    let handle = file_handle(root, name, true).await?;
    let stream = invoke(&handle, "createWritable", &[]).await?;
    let result = async {
        invoke(&stream, "write", &[JsValue::from_str(contents)]).await?;
        invoke(&stream, "close", &[]).await?;
        Ok(())
    }
    .await;
    if result.is_err() {
        let _ = invoke(&stream, "abort", &[]).await;
    }
    result
}

async fn remove_file(root: &JsValue, name: &str) -> Result<(), PersonalDataError> {
    match invoke(root, "removeEntry", &[JsValue::from_str(name)]).await {
        Ok(_) => Ok(()),
        Err(error) if error.to_string().contains("NotFoundError") => Ok(()),
        Err(error) => Err(error),
    }
}

pub async fn status(navigator: &Navigator) -> Result<String, PersonalDataError> {
    let Ok(store) = storage(navigator) else {
        return Ok("Private storage: unavailable".to_owned());
    };
    let estimate = invoke(&store, "estimate", &[]).await.ok();
    let size = |key| {
        estimate
            .as_ref()
            .and_then(|estimate| property(estimate, key).ok())
            .and_then(|value| value.as_f64())
            .map(|value| format!("{} MiB", (value / 1_048_576.0) as u64))
            .unwrap_or_else(|| "unknown".to_owned())
    };
    Ok(format!(
        "Private storage: available · Estimated origin usage: {} · Estimated quota: {}",
        size("usage"),
        size("quota")
    ))
}

pub async fn save(navigator: &Navigator, contents: &str) -> Result<String, PersonalDataError> {
    let root = directory(navigator).await?;
    write_file(&root, SNAPSHOT_FILE, contents).await?;
    Ok("Private browser snapshot saved. Keep a downloaded backup for device loss.".to_owned())
}

pub async fn read(navigator: &Navigator) -> Result<String, PersonalDataError> {
    let root = directory(navigator).await?;
    let contents = read_file(&root, SNAPSHOT_FILE).await?;
    super::backup::parse(&contents)?;
    Ok(contents)
}

pub async fn delete(navigator: &Navigator) -> Result<String, PersonalDataError> {
    let root = directory(navigator).await?;
    remove_file(&root, SNAPSHOT_FILE).await?;
    Ok("Private browser snapshot deleted".to_owned())
}

pub async fn probe(navigator: &Navigator) -> Result<String, PersonalDataError> {
    let root = directory(navigator).await?;
    let token = crate::services::word_sets::WordSetId::generate()
        .map_err(PersonalDataError::Opfs)?
        .to_string();
    let filename = format!("zipeek-opfs-probe-{token}.tmp");
    let contents = format!("zipeek-opfs-roundtrip:{token}");
    let result = async {
        write_file(&root, &filename, &contents).await?;
        let found = read_file(&root, &filename).await?;
        if found != contents {
            return Err(PersonalDataError::Opfs(
                "Private storage readback mismatch".into(),
            ));
        }
        Ok("Private storage write, close, reopen, and read verified for this session".to_owned())
    }
    .await;
    let cleanup = remove_file(&root, &filename).await;
    match (result, cleanup) {
        (Ok(message), Ok(())) => Ok(message),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(PersonalDataError::Opfs(format!(
            "{error}; probe cleanup failed: {cleanup}"
        ))),
    }
}
