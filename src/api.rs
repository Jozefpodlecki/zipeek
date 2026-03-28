use std::{cell::{Ref, RefCell, RefMut}, ops::Deref, rc::Rc};

use js_sys::Uint8Array;
use log::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
#[cfg(debug_assertions)]
use web_sys::RequestCache;
use web_sys::{window, Headers, Request, RequestInit, RequestMode, Response, SpeechSynthesisUtterance, SpeechSynthesisVoice, Window};
use zipseek_core::{HashToLexemeMap, LexemeHash, LexemeNeighbors, OwnedLexemeNeighbors, Shard};

use crate::{models::{AppError, Lexeme, Social}, services::TimestampLruCache};

#[cfg(debug_assertions)]
const CACHE_MODE: RequestCache = RequestCache::NoStore;

#[cfg(not(debug_assertions))]
const CACHE_MODE: RequestCache = RequestCache::Default;

pub struct HttpClient {
    window: Window,
    version: Rc<str>,
    app_name: Rc<str>
}

impl HttpClient {
     pub fn new(window: Window, version: Rc<str>, app_name: Rc<str>) -> Self {
        Self {
            window,
            version,
            app_name
        }
    }

    pub async fn get_as_bytes(&self, url: &str) -> Result<Vec<u8>, AppError> {

        let headers = Headers::new()?;
        headers.set("Cache-Control", "no-cache")?;
        headers.set("X-App-Version", &self.version)?;
        headers.set("X-Client", &self.app_name)?;

        let request_options = RequestInit::new();
        request_options.set_method("GET");
        request_options.set_mode(RequestMode::NoCors);
        request_options.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &request_options)?;

        let response_value = JsFuture::from(self.window.fetch_with_request(&request)).await?;
        let response: Response = response_value.unchecked_into();
        let array_buffer = response.array_buffer().map_err(AppError::failed_to_read_body)?;

        let buffer = JsFuture::from(array_buffer)
            .await
            .map_err(AppError::failed_to_read_body)?;

        let uint8_array = Uint8Array::new(&buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);

        Ok(bytes)
    }
}

#[derive(Clone)]
pub struct HashToLexeme {
    window: Window,
    map: Rc<RefCell<Option<HashToLexemeMap>>>
}

// impl Deref for Test {
//     type Target = HashToLexemeMap;

//     fn deref(&self) -> &Self::Target {
//         &self.map
//     }
// }

impl PartialEq for HashToLexeme {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl HashToLexeme {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            map: Rc::new(RefCell::new(None))
        }
    }

    pub async fn get(&self)  -> Result<Ref<'_, HashToLexemeMap>, AppError> {

        {
            let guard = self.map.borrow();
            if guard.is_some() {
                let map: Ref<'_, HashToLexemeMap> = Ref::map(guard, |pr| pr.as_ref().unwrap());
                return Ok(map)
            }
        }

        {
            let map = self.fetch_index().await?;
            *self.map.borrow_mut() = Some(map);
        }

        let map: Ref<'_, HashToLexemeMap> = Ref::map(self.map.borrow(), |pr| pr.as_ref().unwrap());

        Ok(map)
    }

    async fn fetch_index(&self) -> Result<HashToLexemeMap, AppError> {
        let url = "public/lexicon/index.pb";

        let headers = Headers::new()?;
        headers.set("Cache-Control", "no-cache")?;
        
        let request_options = RequestInit::new();
        request_options.set_method("GET");
        request_options.set_mode(RequestMode::NoCors);
        request_options.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &request_options)?;

        let response_value = JsFuture::from(self.window.fetch_with_request(&request)).await?;
        let response: Response = response_value.unchecked_into();
        let array_buffer = response.array_buffer().map_err(AppError::failed_to_read_body)?;

        let buffer = JsFuture::from(array_buffer)
            .await
            .map_err(AppError::failed_to_read_body)?;

        let uint8_array = Uint8Array::new(&buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);
        
        let map = HashToLexemeMap::decode_from_slice(&bytes)
            .map_err(|err| AppError::failed_to_read_body(err.to_string().into()))?;

        Ok(map)
    }
}

#[derive(Clone, PartialEq)]
pub struct ApiClient {
    window: Window,
    hash_to_lexeme: HashToLexeme,
    shard_map: CachedShardMap
}

impl ApiClient {
    pub fn new(window: Window) -> Self {
        Self {
            window: window.clone(),
            hash_to_lexeme: HashToLexeme::new(window.clone()),
            shard_map: CachedShardMap::new(window)
        }
    }

    pub async fn get_lexeme(&self, lexeme: &str)  -> Result<Option<OwnedLexemeNeighbors>, AppError> {
        let hash_to_lexeme = self.hash_to_lexeme.get().await?;

        let hash = LexemeHash::new(lexeme);
        let lexeme_id = match hash_to_lexeme.get(hash) {
            Some(value) => value,
            None => return Ok(None),
        };

        let shard_count = 50;
        let shard_id = lexeme_id % shard_count;
        let shard: RefMut<'_, Shard> = self.shard_map.get(shard_id).await?;

        let result = shard.get_with_neighbors(lexeme_id).map(|pr| pr.to_owned());

        Ok(result)
    }

    pub async fn get_social(&self) -> Result<Social, AppError> {
        let url = "public/social.json";

        let request_options = RequestInit::new();
        request_options.set_method("GET");
        request_options.set_mode(RequestMode::Cors);
        request_options.set_cache(CACHE_MODE);

        let request = Request::new_with_str_and_init(url, &request_options)
            .map_err(AppError::failed_to_build_request)?;

        let response_value = JsFuture::from(self.window.fetch_with_request(&request))
            .await
            .map_err(AppError::network_request_failed)?;

        let response: Response = response_value.dyn_into()
            .map_err(AppError::invalid_response)?;

        let js_value = JsFuture::from(response.json()?)
            .await
            .map_err(AppError::failed_to_read_body)?;

        let data: Social = serde_wasm_bindgen::from_value(js_value)
            .map_err(AppError::from)?;

        Ok(data)
    }
}


#[derive(Clone)]
pub struct CachedShardMap {
    window: Window,
    map: Rc<RefCell<TimestampLruCache<u64, Shard>>>
}

impl PartialEq for CachedShardMap {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl CachedShardMap {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            map: Rc::new(RefCell::new(TimestampLruCache::new(10)))
        }
    }

    pub async fn get(&self, shard_id: u64)  -> Result<RefMut<'_, Shard>, AppError> {

        {
            let mut guard = self.map.borrow_mut();

            if guard.get(&shard_id).is_some() {
                let shard = RefMut::map(guard, |pr: &mut TimestampLruCache<u64, Shard>|
                    pr.get_mut(&shard_id).unwrap());
                return Ok(shard)
            }
        }

        {
            let mut guard = self.map.borrow_mut();
            let shard = self.fetch_shard(shard_id).await?;
            guard.insert(shard.id, shard);
        }

        let guard = self.map.borrow_mut();
        let shard = RefMut::map(guard, |pr: &mut TimestampLruCache<u64, Shard>|
                pr.get_mut(&shard_id).unwrap());

        Ok(shard)
    }

    async fn fetch_shard(&self, shard_id: u64) -> Result<Shard, AppError> {
        let url = format!("public/lexicon/shard_{}.json", shard_id);
        
        let request_options = RequestInit::new();
        request_options.set_method("GET");
        request_options.set_mode(RequestMode::NoCors);
        
        let headers = Headers::new()?;
        headers.set("Cache-Control", "no-cache")?;
        request_options.set_headers(&headers);
        
        let request = Request::new_with_str_and_init(&url, &request_options)?;
        let response_value = JsFuture::from(self.window.fetch_with_request(&request)).await?;
        let response: Response = response_value.unchecked_into();
        let array_buffer = response.array_buffer().map_err(AppError::failed_to_read_body)?;
        
        let buffer = JsFuture::from(array_buffer)
            .await
            .map_err(AppError::failed_to_read_body)?;
        
        let uint8_array = Uint8Array::new(&buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);
        
        let shard = Shard::decode(bytes)
            .map_err(|err| AppError::failed_to_read_body(err.to_string().into()))?;

        Ok(shard)
    }
}