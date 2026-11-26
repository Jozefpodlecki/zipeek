use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::Instant,
};

use tokio::sync::{mpsc, oneshot, OwnedSemaphorePermit};

use crate::{
    candle_stt::inference::QwenAsrEngine,
    candle_tts::qwen::QwenTtsEngine,
    speech::SpeechError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputeDevice {
    Cpu,
    Cuda,
}

impl ComputeDevice {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Cuda => "cuda",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EngineStatus {
    NotInstalled = 0,
    Installed = 1,
    Loading = 2,
    Warming = 3,
    Ready = 4,
    Failed = 5,
    Stopped = 6,
}

impl EngineStatus {
    fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Installed,
            2 => Self::Loading,
            3 => Self::Warming,
            4 => Self::Ready,
            5 => Self::Failed,
            6 => Self::Stopped,
            _ => Self::NotInstalled,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotInstalled => "not_installed",
            Self::Installed => "installed",
            Self::Loading => "loading",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone)]
struct Status {
    state: Arc<AtomicU8>,
    failure: Arc<AtomicU8>,
}

impl Status {
    fn new(installed: bool) -> Self {
        Self {
            state: Arc::new(AtomicU8::new(if installed { 1 } else { 0 })),
            failure: Arc::new(AtomicU8::new(0)),
        }
    }

    fn get(&self) -> EngineStatus {
        EngineStatus::from_byte(self.state.load(Ordering::Acquire))
    }

    fn set(&self, status: EngineStatus) {
        if status != EngineStatus::Failed {
            self.failure.store(0, Ordering::Release);
        }
        self.state.store(status as u8, Ordering::Release);
    }

    fn fail(&self, kind: u8) {
        self.failure.store(kind, Ordering::Release);
        self.state.store(EngineStatus::Failed as u8, Ordering::Release);
    }

    fn failure_code(&self) -> &'static str {
        match self.failure.load(Ordering::Acquire) {
            1 => "model_load",
            2 => "inference",
            _ => "none",
        }
    }
}

enum TtsRequest {
    Shutdown(oneshot::Sender<()>),
    Warm(oneshot::Sender<Result<(), SpeechError>>),
    Generate {
        text: String,
        cancelled: Arc<AtomicBool>,
        permit: OwnedSemaphorePermit,
        reply: oneshot::Sender<Result<Vec<u8>, SpeechError>>,
    },
}

enum SttRequest {
    Shutdown(oneshot::Sender<()>),
    Warm(oneshot::Sender<Result<(), SpeechError>>),
    Recognize {
        samples: Vec<f32>,
        cancelled: Arc<AtomicBool>,
        permit: OwnedSemaphorePermit,
        reply: oneshot::Sender<Result<String, SpeechError>>,
    },
}

#[derive(Clone)]
pub struct TtsWorker {
    sender: mpsc::Sender<TtsRequest>,
    status: Status,
}

impl TtsWorker {
    pub fn start(model_directory: PathBuf, device: ComputeDevice, installed: bool) -> Self {
        let (sender, mut receiver) = mpsc::channel(1);
        let status = Status::new(installed);
        let worker_status = status.clone();
        std::thread::Builder::new()
            .name("zipeek-qwen-tts".into())
            .spawn(move || {
                let mut engine: Option<QwenTtsEngine> = None;
                while let Some(request) = receiver.blocking_recv() {
                    let initialize = |engine: &mut Option<QwenTtsEngine>, warm: bool| {
                        if engine.is_none() {
                            worker_status.set(EngineStatus::Loading);
                            match QwenTtsEngine::load(&model_directory, device.as_str()) {
                                Ok(loaded) => *engine = Some(loaded),
                                Err(error) => {
                                    worker_status.fail(1);
                                    return Err(SpeechError::ModelLoad(error));
                                }
                            }
                        }
                        if warm {
                            worker_status.set(EngineStatus::Warming);
                            if let Err(error) = engine.as_mut().unwrap().synthesize("你好") {
                                *engine = None;
                                worker_status.fail(2);
                                return Err(SpeechError::Inference(error));
                            }
                        }
                        worker_status.set(EngineStatus::Ready);
                        Ok(())
                    };
                    match request {
                        TtsRequest::Shutdown(reply) => {
                            drop(engine);
                            worker_status.set(EngineStatus::Stopped);
                            let _ = reply.send(());
                            return;
                        }
                        TtsRequest::Warm(reply) => {
                            let started = Instant::now();
                            let result = initialize(&mut engine, true);
                            if result.is_ok() {
                                tracing::info!(elapsed_ms = started.elapsed().as_millis(), "Qwen3-TTS ready after neural warm-up");
                            }
                            let _ = reply.send(result);
                        }
                        TtsRequest::Generate { text, cancelled, permit: _permit, reply } => {
                            let started = !cancelled.load(Ordering::Relaxed);
                            let result = if !started {
                                Err(SpeechError::Cancelled)
                            } else {
                                initialize(&mut engine, false).and_then(|()| {
                                    engine.as_mut().unwrap().synthesize_with_cancel(&text, &cancelled, |_| {})
                                        .map_err(|error| {
                                            if cancelled.load(Ordering::Relaxed) {
                                                SpeechError::Cancelled
                                            } else {
                                                SpeechError::Inference(error)
                                            }
                                        })
                                })
                            };
                            if let Err(error) = &result {
                                match error {
                                    SpeechError::ModelLoad(_) => {}
                                    SpeechError::Cancelled => {
                                        if started && engine.is_some() {
                                            engine = None;
                                            worker_status.set(EngineStatus::Installed);
                                        }
                                    }
                                    _ => {
                                        engine = None;
                                        worker_status.fail(2);
                                    }
                                }
                            }
                            let _ = reply.send(result);
                        }
                    }
                }
                worker_status.set(EngineStatus::Stopped);
            })
            .expect("create Qwen3-TTS worker thread");
        Self { sender, status }
    }

    pub fn status(&self) -> EngineStatus { self.status.get() }

    pub fn failure_code(&self) -> &'static str { self.status.failure_code() }

    pub async fn shutdown(&self) -> Result<(), SpeechError> {
        if self.status() == EngineStatus::Stopped {
            return Ok(());
        }
        let (reply, receiver) = oneshot::channel();
        if self.sender.send(TtsRequest::Shutdown(reply)).await.is_err() {
            return Ok(());
        }
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-TTS shutdown interrupted"))
    }

    pub async fn warm(&self) -> Result<(), SpeechError> {
        let (reply, receiver) = oneshot::channel();
        self.sender.send(TtsRequest::Warm(reply)).await.map_err(|_| SpeechError::Unavailable("Qwen3-TTS worker has shut down"))?;
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-TTS worker stopped during warm-up"))?
    }

    pub async fn generate(&self, text: String, cancelled: Arc<AtomicBool>, permit: OwnedSemaphorePermit) -> Result<Vec<u8>, SpeechError> {
        let (reply, receiver) = oneshot::channel();
        self.sender.try_send(TtsRequest::Generate { text, cancelled, permit, reply }).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => SpeechError::Busy,
            mpsc::error::TrySendError::Closed(_) => SpeechError::Unavailable("Qwen3-TTS worker has shut down"),
        })?;
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-TTS worker stopped"))?
    }
}

#[derive(Clone)]
pub struct SttWorker {
    sender: mpsc::Sender<SttRequest>,
    status: Status,
}

impl SttWorker {
    pub fn start(model: PathBuf, device: ComputeDevice, installed: bool) -> Self {
        let (sender, mut receiver) = mpsc::channel(1);
        let status = Status::new(installed);
        let worker_status = status.clone();
        std::thread::Builder::new()
            .name("zipeek-qwen-stt".into())
            .spawn(move || {
                let mut engine: Option<QwenAsrEngine> = None;
                while let Some(request) = receiver.blocking_recv() {
                    let initialize = |engine: &mut Option<QwenAsrEngine>| {
                        if engine.is_none() {
                            worker_status.set(EngineStatus::Loading);
                            let device = match device {
                                ComputeDevice::Cuda => candle_core::Device::new_cuda(0)
                                    .map_err(|error| format!("Qwen3-ASR CUDA device: {error}")),
                                ComputeDevice::Cpu => Ok(candle_core::Device::Cpu),
                            };
                            let loaded = device.and_then(|device| QwenAsrEngine::load(&model, &device));
                            match loaded {
                                Ok(loaded) => *engine = Some(loaded),
                                Err(error) => {
                                    worker_status.fail(1);
                                    return Err(SpeechError::ModelLoad(error));
                                }
                            }
                        }
                        worker_status.set(EngineStatus::Ready);
                        Ok(())
                    };
                    match request {
                        SttRequest::Shutdown(reply) => {
                            drop(engine);
                            worker_status.set(EngineStatus::Stopped);
                            let _ = reply.send(());
                            return;
                        }
                        SttRequest::Warm(reply) => {
                            let started = Instant::now();
                            let result = initialize(&mut engine);
                            if result.is_ok() {
                                tracing::info!(elapsed_ms = started.elapsed().as_millis(), "Qwen3-ASR weights preloaded");
                            }
                            let _ = reply.send(result);
                        }
                        SttRequest::Recognize { samples, cancelled, permit: _permit, reply } => {
                            let started = !cancelled.load(Ordering::Relaxed);
                            let result = if !started {
                                Err(SpeechError::Cancelled)
                            } else {
                                initialize(&mut engine).and_then(|()| {
                                    engine.as_mut().unwrap().transcribe_with_cancel(&samples, &cancelled)
                                        .map_err(|error| {
                                            if cancelled.load(Ordering::Relaxed) {
                                                SpeechError::Cancelled
                                            } else {
                                                SpeechError::Inference(error)
                                            }
                                        })
                                })
                            };
                            if let Err(error) = &result {
                                match error {
                                    SpeechError::ModelLoad(_) => {}
                                    SpeechError::Cancelled => {
                                        if started && engine.is_some() {
                                            engine = None;
                                            worker_status.set(EngineStatus::Installed);
                                        }
                                    }
                                    _ => {
                                        engine = None;
                                        worker_status.fail(2);
                                    }
                                }
                            }
                            let _ = reply.send(result);
                        }
                    }
                }
                worker_status.set(EngineStatus::Stopped);
            })
            .expect("create Qwen3-ASR worker thread");
        Self { sender, status }
    }

    pub fn status(&self) -> EngineStatus { self.status.get() }

    pub fn failure_code(&self) -> &'static str { self.status.failure_code() }

    pub async fn shutdown(&self) -> Result<(), SpeechError> {
        if self.status() == EngineStatus::Stopped {
            return Ok(());
        }
        let (reply, receiver) = oneshot::channel();
        if self.sender.send(SttRequest::Shutdown(reply)).await.is_err() {
            return Ok(());
        }
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-ASR shutdown interrupted"))
    }

    pub async fn warm(&self) -> Result<(), SpeechError> {
        let (reply, receiver) = oneshot::channel();
        self.sender.send(SttRequest::Warm(reply)).await.map_err(|_| SpeechError::Unavailable("Qwen3-ASR worker has shut down"))?;
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-ASR worker stopped during preload"))?
    }

    pub async fn recognize(&self, samples: Vec<f32>, cancelled: Arc<AtomicBool>, permit: OwnedSemaphorePermit) -> Result<String, SpeechError> {
        let (reply, receiver) = oneshot::channel();
        self.sender.try_send(SttRequest::Recognize { samples, cancelled, permit, reply }).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => SpeechError::Busy,
            mpsc::error::TrySendError::Closed(_) => SpeechError::Unavailable("Qwen3-ASR worker has shut down"),
        })?;
        receiver.await.map_err(|_| SpeechError::Unavailable("Qwen3-ASR worker stopped"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_transitions_are_shared_between_clones() {
        let original = Status::new(true);
        let cloned = original.clone();
        assert_eq!(cloned.get(), EngineStatus::Installed);
        original.set(EngineStatus::Loading);
        assert_eq!(cloned.get(), EngineStatus::Loading);
        cloned.set(EngineStatus::Ready);
        assert_eq!(original.get(), EngineStatus::Ready);
    }

    #[tokio::test]
    async fn missing_tts_checkpoint_fails_without_claiming_readiness() {
        let worker = TtsWorker::start(PathBuf::from("missing-tts-model"), ComputeDevice::Cpu, true);
        assert_eq!(worker.status(), EngineStatus::Installed);
        assert!(worker.warm().await.is_err());
        assert_eq!(worker.status(), EngineStatus::Failed);
    }

    #[tokio::test]
    async fn missing_asr_checkpoint_fails_without_claiming_readiness() {
        let worker = SttWorker::start(PathBuf::from("missing-asr-model.safetensors"), ComputeDevice::Cpu, true);
        assert_eq!(worker.status(), EngineStatus::Installed);
        assert!(worker.warm().await.is_err());
        assert_eq!(worker.status(), EngineStatus::Failed);
    }

    #[test]
    fn status_labels_are_stable() {
        assert_eq!(EngineStatus::NotInstalled.as_str(), "not_installed");
        assert_eq!(EngineStatus::Warming.as_str(), "warming");
        assert_eq!(EngineStatus::Failed.as_str(), "failed");
    }
    #[test]
    fn failed_state_exposes_category_without_internal_details() {
        let status = Status::new(true);
        status.fail(1);
        assert_eq!(status.get(), EngineStatus::Failed);
        assert_eq!(status.failure_code(), "model_load");
        status.set(EngineStatus::Loading);
        assert_eq!(status.failure_code(), "none");
        status.fail(2);
        assert_eq!(status.failure_code(), "inference");
        status.set(EngineStatus::Ready);
        assert_eq!(status.failure_code(), "none");
    }

    #[tokio::test]
    async fn cancelled_before_inference_keeps_tts_worker_usable() {
        let worker = TtsWorker::start(PathBuf::from("missing-tts-model"), ComputeDevice::Cpu, true);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(1));
        for _ in 0..20 {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let cancelled = Arc::new(AtomicBool::new(true));
            let result = worker.generate("你好".into(), cancelled, permit).await;
            assert!(matches!(result, Err(SpeechError::Cancelled)));
            assert_eq!(worker.status(), EngineStatus::Installed);
            assert_eq!(worker.failure_code(), "none");
        }
        assert!(semaphore.try_acquire().is_ok());
    }

    #[tokio::test]
    async fn cancelled_before_inference_keeps_stt_worker_usable() {
        let worker = SttWorker::start(PathBuf::from("missing-asr-model"), ComputeDevice::Cpu, true);
        let permit = Arc::new(tokio::sync::Semaphore::new(1))
            .acquire_owned().await.unwrap();
        let result = worker.recognize(vec![0.2], Arc::new(AtomicBool::new(true)), permit).await;
        assert!(matches!(result, Err(SpeechError::Cancelled)));
        assert_eq!(worker.status(), EngineStatus::Installed);
    }

    #[tokio::test]
    async fn worker_owned_permit_outlives_abandoned_request() {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(1));
        let (sent, mut queued) = mpsc::channel::<OwnedSemaphorePermit>(1);
        let (release, released) = oneshot::channel::<()>();
        let worker = tokio::spawn(async move {
            let permit = queued.recv().await.unwrap();
            let _ = released.await;
            drop(permit);
        });
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        sent.send(permit).await.unwrap();
        assert!(semaphore.try_acquire().is_err());
        drop(sent);
        release.send(()).unwrap();
        worker.await.unwrap();
        assert!(semaphore.try_acquire().is_ok());
    }

    #[tokio::test]
    async fn bounded_worker_queue_rejects_excess_requests() {
        let (sender, _receiver) = mpsc::channel::<u8>(1);
        assert!(sender.try_send(1).is_ok());
        assert!(matches!(sender.try_send(2), Err(mpsc::error::TrySendError::Full(2))));
    }

    #[tokio::test]
    async fn shutdown_closes_both_workers_without_checkpoints() {
        let tts = TtsWorker::start(PathBuf::from("absent-tts"), ComputeDevice::Cpu, false);
        let stt = SttWorker::start(PathBuf::from("absent-stt"), ComputeDevice::Cpu, false);
        let (tts_result, stt_result) = tokio::join!(tts.shutdown(), stt.shutdown());
        tts_result.unwrap();
        stt_result.unwrap();
        assert_eq!(tts.status(), EngineStatus::Stopped);
        assert_eq!(stt.status(), EngineStatus::Stopped);
        assert!(matches!(tts.warm().await, Err(SpeechError::Unavailable(_))));
        assert!(matches!(stt.warm().await, Err(SpeechError::Unavailable(_))));
    }

}
