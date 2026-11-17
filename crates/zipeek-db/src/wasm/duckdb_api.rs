use crate::load_module;
use js_sys::{Array, Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "DuckDBModule")]
    pub type DuckDbModule;

    #[wasm_bindgen(typescript_type = "AsyncDuckDB")]
    pub type AsyncDuckDb;

    #[wasm_bindgen(typescript_type = "VoidLogger")]
    pub type VoidLogger;

    #[wasm_bindgen(typescript_type = "AsyncDuckDBConnection")]
    pub type DuckDbConnection;

    #[wasm_bindgen(typescript_type = "AsyncPreparedStatement")]
    pub type DuckDbPreparedStatement;

    #[wasm_bindgen(method, catch, js_name = selectBundle)]
    pub async fn select_bundle(
        this: &DuckDbModule,
        candidates: &JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = instantiate)]
    pub async fn instantiate(this: &AsyncDuckDb, module_url: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = registerFileBuffer)]
    pub async fn register_file_buffer(
        this: &AsyncDuckDb,
        filename: &str,
        bytes: &Uint8Array,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = open)]
    pub async fn open(this: &AsyncDuckDb, options: &JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = connect)]
    pub async fn connect(this: &AsyncDuckDb) -> Result<DuckDbConnection, JsValue>;

    #[wasm_bindgen(method, catch, js_name = terminate)]
    pub async fn terminate(this: &AsyncDuckDb) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = prepare)]
    pub async fn prepare(
        this: &DuckDbConnection,
        sql: &str,
    ) -> Result<DuckDbPreparedStatement, JsValue>;

    #[wasm_bindgen(method, catch, js_name = query)]
    pub async fn query(this: &DuckDbConnection, sql: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = close)]
    pub async fn close_connection(this: &DuckDbConnection) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = close)]
    pub async fn close_statement(this: &DuckDbPreparedStatement) -> Result<(), JsValue>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
}

impl AccessMode {
    fn javascript_name(self) -> &'static str {
        match self {
            Self::ReadOnly => "READ_ONLY",
            Self::ReadWrite => "READ_WRITE",
        }
    }
}

fn construct(module: &JsValue, constructor: &str, args: &Array) -> Result<JsValue, JsValue> {
    let class = Reflect::get(module, &JsValue::from_str(constructor))?
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str("DuckDB constructor is not callable"))?;
    Reflect::construct(&class, args)
}

impl DuckDbModule {
    pub async fn load(url: &str) -> Result<Self, JsValue> {
        Ok(crate::load_module(url).await?.unchecked_into())
    }

    pub fn open_options(&self, path: &str, mode: AccessMode) -> Result<JsValue, JsValue> {
        let access_modes = Reflect::get(self.as_ref(), &JsValue::from_str("DuckDBAccessMode"))?;
        let access_mode = Reflect::get(&access_modes, &JsValue::from_str(mode.javascript_name()))?;
        if access_mode.is_undefined() {
            return Err(JsValue::from_str("DuckDB access mode is unavailable"));
        }
        let options = js_sys::Object::new();
        Reflect::set(
            &options,
            &JsValue::from_str("path"),
            &JsValue::from_str(path),
        )?;
        Reflect::set(&options, &JsValue::from_str("accessMode"), &access_mode)?;
        Ok(options.into())
    }

    pub fn void_logger(&self) -> Result<VoidLogger, JsValue> {
        Ok(construct(self.as_ref(), "VoidLogger", &Array::new())?.unchecked_into())
    }

    pub fn async_duck_db(
        &self,
        logger: &VoidLogger,
        worker: &JsValue,
    ) -> Result<AsyncDuckDb, JsValue> {
        let args = Array::new();
        args.push(logger.as_ref());
        args.push(worker);
        Ok(construct(self.as_ref(), "AsyncDuckDB", &args)?.unchecked_into())
    }
}

impl DuckDbPreparedStatement {
    pub async fn query_parameters(&self, parameters: &Array) -> Result<JsValue, JsValue> {
        let query = Reflect::get(self.as_ref(), &JsValue::from_str("query"))?
            .dyn_into::<Function>()
            .map_err(|_| JsValue::from_str("Prepared statement query method is not callable"))?;
        let result = query.apply(self.as_ref(), parameters)?;
        let promise = result.dyn_into::<Promise>()?;
        wasm_bindgen_futures::JsFuture::from(promise).await
    }
}
