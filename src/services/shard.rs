use std::{cell::{RefCell, RefMut}, rc::Rc};

use js_sys::Uint8Array;
use log::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[cfg(debug_assertions)]
use web_sys::RequestCache;

use web_sys::{Headers, Request, RequestInit, RequestMode, Response, Window};
use zipseek_core::Shard;

use crate::{models::AppError, services::TimestampLruCache};

#[derive(Clone)]
pub struct CachedShardMap {
    window: Window,
    map: Rc<RefCell<TimestampLruCache<u64, Shard>>>
}

impl PartialEq for CachedShardMap {
    fn eq(&self, _: &Self) -> bool {
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