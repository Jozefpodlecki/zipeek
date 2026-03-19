use log::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response, Window};
use xxhash_rust::xxh3::xxh3_64;

use crate::models::{FetchError, Lexeme};

pub struct ApiClient(Window);

impl ApiClient {
    pub fn new() -> Self {
        Self(gloo::utils::window())
    }

    pub async fn get_shard(&self, lexeme: &str)  -> Result<Vec<Lexeme>, FetchError> {
        unsafe {
            let shard_id = xxh3_64(lexeme.as_bytes());
            let url = format!("public/shard_{}.json", shard_id);
            
            let request_options = RequestInit::new();
            request_options.set_method("GET");
            request_options.set_mode(RequestMode::NoCors);

            let headers = Headers::new().unwrap_unchecked();
            headers.set("Cache-Control", "no-cache").unwrap_unchecked();
            request_options.set_headers(&headers);

            let request = Request::new_with_str_and_init(&url, &request_options)?;

            let response_value = JsFuture::from(self.0.fetch_with_request(&request)).await?;
            let response: Response = response_value.dyn_into().unwrap();

            let js_value = JsFuture::from(response.json()?).await?;
            let items: Vec<Lexeme> = serde_wasm_bindgen::from_value(js_value)?;

            Ok(items)
        }
    }
}