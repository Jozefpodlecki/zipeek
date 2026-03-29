use std::{cell::RefMut, rc::Rc};
use web_sys::*;
use log::*;
use zipseek_core::{ChineseLexeme, LexemeHash, OwnedLexemeNeighbors, Shard};

use crate::{models::{AppError, Social}, services::{CachedShardMap, HashToLexeme, HttpClient, SearchIndexClient}};

#[cfg(debug_assertions)]
const CACHE_MODE: RequestCache = RequestCache::NoStore;

#[cfg(not(debug_assertions))]
const CACHE_MODE: RequestCache = RequestCache::Default;

#[derive(Clone, PartialEq)]
pub struct ApiClient {
    window: Window,
    http_client: HttpClient,
    search_index: SearchIndexClient,
    hash_to_lexeme: HashToLexeme,
    shard_map: CachedShardMap
}

impl ApiClient {
    pub fn new(window: Window, http_client: HttpClient, version: Rc<str>, app_name: Rc<str>) -> Self {
        Self {
            window: window.clone(),
            http_client: http_client.clone(),
            search_index: SearchIndexClient::new(window.clone()),
            hash_to_lexeme: HashToLexeme::new(window.clone()),
            shard_map: CachedShardMap::new(http_client)
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

    pub async fn search(&self, phrase: &str) -> Result<Vec<ChineseLexeme>, AppError> {

        let search_index = self.search_index.get().await?;
        let hash_to_lexeme = self.hash_to_lexeme.get().await?;

        let lexeme_ids = search_index.search(phrase);
        let mut items = vec![];
        
        for lexeme_id in lexeme_ids {
            let shard_id = hash_to_lexeme.get_shard_id(lexeme_id);
             info!("{} {}", lexeme_id, shard_id);
            let shard = self.shard_map.get(shard_id).await?;
            info!("test2");
            let lexeme = shard.get(&lexeme_id);
            info!("test3");
            if let Some(lexeme) = lexeme {
                items.push(lexeme.clone());
            }
        }

        Ok(items)
    }

    pub async fn get_social(&self) -> Result<Social, AppError> {
        let url = "public/social.json";
        let data = self.http_client.get_as_json(url).await?;

        Ok(data)
    }
}