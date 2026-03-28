use std::{cell::RefMut, rc::Rc};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use web_sys::*;

use zipseek_core::{LexemeHash, OwnedLexemeNeighbors, Shard};

use crate::{models::{AppError, Social}, services::{CachedShardMap, HashToLexeme, HttpClient}};

#[cfg(debug_assertions)]
const CACHE_MODE: RequestCache = RequestCache::NoStore;

#[cfg(not(debug_assertions))]
const CACHE_MODE: RequestCache = RequestCache::Default;

#[derive(Clone, PartialEq)]
pub struct ApiClient {
    window: Window,
    http_client: HttpClient,
    hash_to_lexeme: HashToLexeme,
    shard_map: CachedShardMap
}

impl ApiClient {
    pub fn new(window: Window, version: Rc<str>, app_name: Rc<str>) -> Self {
        Self {
            window: window.clone(),
            http_client: HttpClient::new(window.clone(), version, app_name),
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

        let shard: RefMut<'_, Shard> = self.shard_map.get(lexeme_id.shard_id).await?;

        let result = shard.get_with_neighbors(lexeme_id).map(|pr| pr.to_owned());

        Ok(result)
    }

    pub async fn get_social(&self) -> Result<Social, AppError> {
        let url = "public/social.json";
        let data = self.http_client.get_as_json(url).await?;

        Ok(data)
    }
}