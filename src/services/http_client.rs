use std::rc::Rc;

use js_sys::Uint8Array;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use web_sys::*;
use zipseek_core::ProtobufDeserialize;

use crate::models::AppError;

#[cfg(debug_assertions)]
const CACHE_MODE: RequestCache = RequestCache::NoStore;

#[cfg(not(debug_assertions))]
const CACHE_MODE: RequestCache = RequestCache::Default;

pub struct HttpResponse(Response);

impl HttpResponse {
    pub async fn into_json<T: DeserializeOwned>(self) -> Result<T, AppError> {
        let js_value = JsFuture::from(self.0.json()?)
            .await
            .map_err(AppError::failed_to_read_body)?;
        
        serde_wasm_bindgen::from_value(js_value)
            .map_err(AppError::from)
    }

    pub async fn into_protobuf<T: ProtobufDeserialize>(self) -> Result<T, AppError> {
        let bytes = self.into_bytes().await?;
        
        T::decode_from_slice(&bytes)
            .map_err(|err| AppError::failed_to_read_body(err.to_string().into()))
    }
    
    pub async fn into_bytes(self) -> Result<Vec<u8>, AppError> {
        let array_buffer = self.0.array_buffer()
            .map_err(AppError::failed_to_read_body)?;
        
        let buffer = JsFuture::from(array_buffer)
            .await
            .map_err(AppError::failed_to_read_body)?;
        
        let uint8_array = Uint8Array::new(&buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);
        
        Ok(bytes)
    }
}

#[derive(Clone, PartialEq)]
pub struct HttpClient {
    window: Window,
    headers: Headers,
    version: Rc<str>,
    app_name: Rc<str>
}

impl HttpClient {
    pub fn new(window: Window, version: Rc<str>, app_name: Rc<str>) -> Self {
        let headers = Headers::new().unwrap();

        Self {
            window,
            headers,
            version,
            app_name
        }
    }

    fn build_headers(&self) -> Result<Headers, AppError> {
        let headers = Headers::new()?;
        headers.set("Cache-Control", "no-cache")?;
        headers.set("X-App-Version", &self.version)?;
        headers.set("X-Client", &self.app_name)?;
        Ok(headers)
    }

    fn create_request(&self, url: &str, method: &str, mode: RequestMode, include_headers: bool) -> Result<Request, AppError> {
        let request_options = RequestInit::new();
        request_options.set_method(method);
        request_options.set_mode(mode);
        request_options.set_cache(CACHE_MODE);
        
        if include_headers {
            request_options.set_headers(&self.build_headers()?.into());
        }
        
        Request::new_with_str_and_init(url, &request_options)
            .map_err(AppError::failed_to_build_request)
    }

    async fn execute_request(&self, request: Request) -> Result<HttpResponse, AppError> {
        let response_value = JsFuture::from(self.window.fetch_with_request(&request))
            .await
            .map_err(AppError::network_request_failed)?;
        
        let response: Response = response_value
            .dyn_into()
            .map_err(AppError::invalid_response)?;
        
        Ok(HttpResponse(response))
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse, AppError> {
        let request = self.create_request(url, "GET", RequestMode::NoCors, true)?;
        self.execute_request(request).await
    }
    
    pub async fn get_with_cors(&self, url: &str) -> Result<HttpResponse, AppError> {
        let request = self.create_request(url, "GET", RequestMode::Cors, false)?;
        self.execute_request(request).await
    }
    
    pub async fn get_as_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, AppError> {
        self.get_with_cors(url).await?.into_json().await
    }

    pub async fn get_as_protobuf<T: ProtobufDeserialize>(&self, url: &str) -> Result<T, AppError> {
        self.get_with_cors(url).await?.into_protobuf().await
    }
    
    pub async fn get_as_bytes(&self, url: &str) -> Result<Vec<u8>, AppError> {
        self.get(url).await?.into_bytes().await
    }
}
