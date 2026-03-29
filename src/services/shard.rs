use std::{cell::{RefCell, RefMut}, rc::Rc};

use js_sys::Uint8Array;
use log::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[cfg(debug_assertions)]
use web_sys::RequestCache;

use web_sys::{Headers, Request, RequestInit, RequestMode, Response, Window};
use zipseek_core::Shard;

use crate::{models::AppError, services::{HttpClient, TimestampLruCache}};

#[derive(Clone)]
pub struct CachedShardMap {
    client: HttpClient,
    map: Rc<RefCell<TimestampLruCache<u64, Shard>>>
}

impl PartialEq for CachedShardMap {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl CachedShardMap {
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
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
        
        let shard = self.client.get_as_protobuf(&url).await?;

        Ok(shard)
    }
}