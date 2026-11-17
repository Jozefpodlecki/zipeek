use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};

use web_sys::Worker;

use super::DbError;
use super::duckdb_api::{AccessMode, AsyncDuckDb, DuckDbConnection, DuckDbModule};
use super::error::js_error;

struct WorkerGuard {
    worker: Option<Worker>,
    database: Option<AsyncDuckDb>,
}

impl WorkerGuard {
    fn new(url: &str) -> Result<Self, DbError> {
        Worker::new(url)
            .map(|worker| Self {
                worker: Some(worker),
                database: None,
            })
            .map_err(|error| js_error("Creating DuckDB worker", error))
    }

    fn worker(&self) -> &Worker {
        self.worker
            .as_ref()
            .expect("Worker guard must own its worker")
    }

    fn track(&mut self, database: &AsyncDuckDb) {
        let value: &JsValue = database.as_ref();
        self.database = Some(value.clone().unchecked_into());
    }

    fn release(mut self) -> Worker {
        self.database.take();
        self.worker
            .take()
            .expect("Worker guard must own its worker")
    }

    fn untrack(&mut self) {
        self.database.take();
    }
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.terminate();
        }
        if let Some(db) = self.database.take() {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(error) = db.terminate().await {
                    log::warn!("Cancelled database initialization cleanup failed: {error:?}");
                }
            });
        }
    }
}

pub struct DbSession {
    pub db: AsyncDuckDb,
    pub connection: DuckDbConnection,
    worker: Worker,
    closed: bool,
}

impl DbSession {
    pub async fn open(
        module: &DuckDbModule,
        main_module: &str,
        main_worker: &str,
        path: &str,
        access_mode: AccessMode,
        file_bytes: Option<&Uint8Array>,
    ) -> Result<Self, DbError> {
        let mut worker = WorkerGuard::new(main_worker)?;
        let db = match module
            .void_logger()
            .and_then(|logger| module.async_duck_db(&logger, worker.worker().as_ref()))
        {
            Ok(db) => db,
            Err(error) => {
                return Err(js_error("Constructing DuckDB instance", error));
            }
        };
        worker.track(&db);
        let initialized = async {
            db.instantiate(main_module)
                .await
                .map_err(|e| js_error("Instantiating DuckDB", e))?;
            if let Some(bytes) = file_bytes {
                db.register_file_buffer(path, bytes)
                    .await
                    .map_err(|e| js_error("Registering dictionary file", e))?;
            }
            let options = module
                .open_options(path, access_mode)
                .map_err(|e| js_error("Configuring DuckDB open options", e))?;
            db.open(&options)
                .await
                .map_err(|e| js_error("Opening DuckDB", e))?;
            db.connect()
                .await
                .map_err(|e| js_error("Connecting DuckDB", e))
        }
        .await;
        match initialized {
            Ok(connection) => Ok(Self {
                db,
                connection,
                worker: worker.release(),
                closed: false,
            }),
            Err(error) => {
                if let Err(cleanup_error) = db.terminate().await {
                    log::warn!("DuckDB initialization cleanup failed: {cleanup_error:?}");
                }
                worker.untrack();
                Err(error)
            }
        }
    }

    pub async fn close(mut self) -> Result<(), DbError> {
        let connection_result = self
            .connection
            .close_connection()
            .await
            .map_err(|error| js_error("Closing DuckDB connection", error));
        let termination_result = self
            .db
            .terminate()
            .await
            .map_err(|error| js_error("Terminating DuckDB", error));
        self.worker.terminate();
        self.closed = true;
        match (connection_result, termination_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(connection), Ok(())) => Err(connection),
            (Ok(()), Err(termination)) => Err(termination),
            (Err(connection), Err(termination)) => Err(DbError::Operation {
                operation: "Closing DuckDB session",
                message: format!(
                    "Connection close failed: {connection}; termination also failed: {termination}"
                ),
            }),
        }
    }
}

impl Drop for DbSession {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        let connection_value: &JsValue = self.connection.as_ref();
        let connection: DuckDbConnection = connection_value.clone().unchecked_into();
        let db_value: &JsValue = self.db.as_ref();
        let db: AsyncDuckDb = db_value.clone().unchecked_into();
        let worker = self.worker.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(error) = connection.close_connection().await {
                log::warn!("DuckDB connection cleanup failed: {error:?}");
            }
            if let Err(error) = db.terminate().await {
                log::warn!("DuckDB termination failed: {error:?}");
            }
            worker.terminate();
        });
    }
}
