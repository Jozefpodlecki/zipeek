use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AbortSignal, Blob, Document, ReadableStream, Request, RequestCache, RequestInit, Response,
    Window,
};

use super::DbError;
use super::error::js_error;
use super::manifest::{DictionaryAsset, DictionaryManifest};

async fn decompress_gzip(bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    use super::browser_api::{args, call, constructor};

    let parts = args(&[Uint8Array::from(bytes).into()]);
    let blob = Blob::new_with_u8_array_sequence(parts.as_ref())?;
    let decoder = constructor("DecompressionStream", &args(&[JsValue::from_str("gzip")]))?;
    let decoded = call(
        blob.stream().as_ref(),
        "pipeThrough",
        &args(&[decoder.into()]),
    )?
    .dyn_into::<ReadableStream>()
    .map_err(|_| JsValue::from_str("Gzip decoder did not return a readable stream"))?;
    let response = Response::new_with_opt_readable_stream(Some(&decoded))?;
    let data = JsFuture::from(response.array_buffer()?).await?;
    Ok(Uint8Array::new(&data).to_vec())
}

pub struct DownloadedDictionary {
    pub filename: String,
    pub native_engine: String,
    pub storage_compatibility: String,
    pub bytes: Uint8Array,
}

pub async fn download_dictionary(
    document: &Document,
    window: &Window,
) -> Result<DownloadedDictionary, DbError> {
    let base_uri = js_sys::Reflect::get(document.as_ref(), &JsValue::from_str("baseURI"))
        .map_err(|e| js_error("Reading document base URI", e))?
        .as_string()
        .ok_or_else(|| DbError::Decode("Document base URI unavailable".to_owned()))?;
    let manifest_url = web_sys::Url::new_with_base("hanzi.manifest.json", &base_uri)
        .map_err(|e| js_error("Resolving manifest URL", e))?;
    let manifest_bytes = fetch_bytes(
        window,
        &manifest_url.href(),
        Some(RequestCache::NoStore),
        30_000,
        "Downloading dictionary manifest",
    )
    .await?;
    let manifest: DictionaryManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| DbError::Decode(format!("Invalid dictionary manifest: {e}")))?;
    let selected = manifest.select()?;
    let filename = selected.filename.to_owned();
    let artifact = selected.download.clone();
    let original = selected.original.clone();
    let compressed = selected.compressed;
    let asset_url = web_sys::Url::new_with_base(&artifact.path, &base_uri)
        .map_err(|e| js_error("Resolving dictionary asset URL", e))?;
    let wire = fetch_bytes(
        window,
        &asset_url.href(),
        None,
        120_000,
        "Downloading dictionary asset",
    )
    .await?;
    verify_integrity(window, &wire, &artifact, "Dictionary download").await?;
    let bytes = if compressed {
        let decoded = decompress_gzip(&wire)
            .await
            .map_err(|e| js_error("Decompressing dictionary", e))?;
        verify_integrity(window, &decoded, &original, "Decompressed dictionary").await?;
        decoded
    } else {
        wire
    };
    Ok(DownloadedDictionary {
        filename,
        native_engine: manifest.engine,
        storage_compatibility: manifest.storage_compatibility,
        bytes: Uint8Array::from(bytes.as_slice()),
    })
}

async fn fetch_bytes(
    window: &Window,
    url: &str,
    cache: Option<RequestCache>,
    timeout_ms: u32,
    operation: &'static str,
) -> Result<Vec<u8>, DbError> {
    let options = RequestInit::new();
    if let Some(cache) = cache {
        options.set_cache(cache);
    }
    let deadline = AbortSignal::timeout_with_u32(timeout_ms);
    options.set_signal(Some(&deadline));
    let request = Request::new_with_str_and_init(url, &options)
        .map_err(|e| js_error("Creating dictionary fetch request", e))?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| js_error(operation, e))?
        .dyn_into::<Response>()
        .map_err(|e| js_error("Decoding dictionary fetch response", e))?;
    if !response.ok() {
        return Err(DbError::Operation {
            operation,
            message: format!("HTTP {}", response.status()),
        });
    }
    let buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|e| js_error(operation, e))?,
    )
    .await
    .map_err(|e| js_error(operation, e))?
    .dyn_into::<ArrayBuffer>()
    .map_err(|e| js_error("Reading dictionary response bytes", e))?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

async fn verify_integrity(
    window: &Window,
    bytes: &[u8],
    expected: &DictionaryAsset,
    operation: &'static str,
) -> Result<(), DbError> {
    if bytes.len() as u64 != expected.size_bytes {
        return Err(DbError::Operation {
            operation,
            message: format!("Size mismatch for {}", expected.path),
        });
    }
    let crypto = window
        .crypto()
        .map_err(|e| js_error("Accessing browser crypto", e))?;
    let digest = JsFuture::from(
        crypto
            .subtle()
            .digest_with_str_and_u8_array("SHA-256", bytes)
            .map_err(|e| js_error("Hashing dictionary asset", e))?,
    )
    .await
    .map_err(|e| js_error("Hashing dictionary asset", e))?;
    let digest_bytes = Uint8Array::new(&digest).to_vec();
    let actual = digest_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if !actual.eq_ignore_ascii_case(&expected.sha256) {
        return Err(DbError::Operation {
            operation,
            message: format!("Checksum mismatch for {}", expected.path),
        });
    }
    Ok(())
}
