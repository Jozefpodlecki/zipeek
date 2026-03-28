use std::{cell::{Ref, RefCell, RefMut}, ops::Deref, rc::Rc};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use web_sys::*;
use zipseek_core::HashToLexemeMap;

use crate::models::AppError;

#[derive(Clone)]
pub struct HashToLexeme {
    window: Window,
    map: Rc<RefCell<Option<HashToLexemeMap>>>
}

impl PartialEq for HashToLexeme {
    fn eq(&self, _: &Self) -> bool {
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