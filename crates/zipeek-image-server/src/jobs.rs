use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    inference::ImageBackend,
    storage::{ImageStore, StoredImage},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageJob {
    pub id: Uuid,
    pub status: JobStatus,
    pub image_url: Option<String>,
    pub error: Option<String>,
}

pub struct JobCoordinator {
    backend: Arc<dyn ImageBackend>,
    store: ImageStore,
    jobs: Mutex<HashMap<Uuid, ImageJob>>,
    slots: Arc<Semaphore>,
    worker: Arc<Semaphore>,
    backend_name: &'static str,
}

impl JobCoordinator {
    pub fn new(
        backend: Arc<dyn ImageBackend>,
        store: ImageStore,
        capacity: usize,
        backend_name: &'static str,
    ) -> Self {
        Self {
            backend,
            store,
            jobs: Mutex::new(HashMap::new()),
            slots: Arc::new(Semaphore::new(capacity)),
            worker: Arc::new(Semaphore::new(1)),
            backend_name,
        }
    }

    pub fn backend_name(&self) -> &'static str {
        self.backend_name
    }

    pub fn get_job(&self, id: Uuid) -> Option<ImageJob> {
        self.jobs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned()
    }

    fn update(
        &self,
        id: Uuid,
        status: JobStatus,
        image_url: Option<String>,
        error: Option<String>,
    ) {
        let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(job) = jobs.get_mut(&id) {
            job.status = status;
            job.image_url = image_url;
            job.error = error;
        }
    }

    pub fn submit(self: &Arc<Self>, prompt: String, width: u32, height: u32) -> Option<ImageJob> {
        let permit = self.slots.clone().try_acquire_owned().ok()?;
        let job = ImageJob {
            id: Uuid::new_v4(),
            status: JobStatus::Queued,
            image_url: None,
            error: None,
        };
        self.jobs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(job.id, job.clone());
        let id = job.id;
        let coordinator = Arc::clone(self);
        tokio::spawn(async move {
            let _slot = permit;
            let Ok(_worker) = coordinator.worker.clone().acquire_owned().await else {
                return;
            };
            coordinator.update(id, JobStatus::Running, None, None);
            let result = coordinator.backend.generate(&prompt, width, height).await;
            match result {
                Ok(image) if matches!(image.media_type, "image/svg+xml" | "image/png") => {
                    let stored = StoredImage {
                        id,
                        media_type: image.media_type.into(),
                        prompt,
                        width,
                        height,
                    };
                    match coordinator.store.put(&stored, &image.bytes).await {
                        Ok(()) => coordinator.update(
                            id,
                            JobStatus::Completed,
                            Some(format!("/v1/images/{id}")),
                            None,
                        ),
                        Err(err) => {
                            tracing::error!(%id, %err, "cannot persist generated image");
                            coordinator.update(
                                id,
                                JobStatus::Failed,
                                None,
                                Some("image storage failed".into()),
                            );
                        }
                    }
                }
                Ok(_) => coordinator.update(
                    id,
                    JobStatus::Failed,
                    None,
                    Some("unsupported image format".into()),
                ),
                Err(err) => {
                    tracing::warn!(%id, %err, "image backend failed");
                    coordinator.update(
                        id,
                        JobStatus::Failed,
                        None,
                        Some("image generation failed".into()),
                    );
                }
            }
        });
        Some(job)
    }

    pub async fn get_image(&self, id: Uuid) -> std::io::Result<Option<(StoredImage, Vec<u8>)>> {
        self.store.get(id).await
    }
}
