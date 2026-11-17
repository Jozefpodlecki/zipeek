#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

use std::{
    error::Error,
    fmt,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenerationOptions {
    pub max_output_tokens: u32,
    pub temperature: f32,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            max_output_tokens: 512,
            temperature: 0.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenerationRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub options: GenerationOptions,
}

impl GenerationRequest {
    pub fn validate(&self) -> Result<(), GenerationError> {
        if self.model.trim().is_empty() {
            return Err(GenerationError::InvalidRequest("model identifier is empty"));
        }
        if self.messages.is_empty()
            || self
                .messages
                .iter()
                .any(|message| message.content.trim().is_empty())
        {
            return Err(GenerationError::InvalidRequest(
                "messages must contain nonempty content",
            ));
        }
        if self.options.max_output_tokens == 0 {
            return Err(GenerationError::InvalidRequest(
                "max_output_tokens must be positive",
            ));
        }
        if !self.options.temperature.is_finite() || !(0.0..=2.0).contains(&self.options.temperature)
        {
            return Err(GenerationError::InvalidRequest(
                "temperature must be between 0 and 2",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationEvent {
    Content(String),
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    InvalidRequest(&'static str),
    ModelUnavailable(String),
    Cancelled,
    Backend(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(f, "invalid generation request: {reason}"),
            Self::ModelUnavailable(model) => write!(f, "model unavailable: {model}"),
            Self::Cancelled => f.write_str("generation cancelled"),
            Self::Backend(reason) => write!(f, "generation backend failed: {reason}"),
        }
    }
}

impl Error for GenerationError {}

#[derive(Clone, Debug, Default)]
pub struct GenerationCancellation(Arc<AtomicBool>);

impl GenerationCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

pub type GenerationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), GenerationError>> + Send + 'a>>;

pub trait GenerationBackend: Send + Sync {
    fn generate<'a>(
        &'a self,
        request: GenerationRequest,
        cancellation: GenerationCancellation,
        on_event: &'a mut (dyn FnMut(GenerationEvent) + Send),
    ) -> GenerationFuture<'a>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GenerationRequest {
        GenerationRequest {
            model: "local-model".to_owned(),
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "你好".to_owned(),
            }],
            options: GenerationOptions::default(),
        }
    }

    #[test]
    fn validates_model_messages_and_generation_bounds() {
        assert!(request().validate().is_ok());
        let mut invalid = request();
        invalid.model = " ".into();
        assert!(invalid.validate().is_err());
        invalid = request();
        invalid.messages.clear();
        assert!(invalid.validate().is_err());
        invalid = request();
        invalid.messages[0].content.clear();
        assert!(invalid.validate().is_err());
        invalid = request();
        invalid.options.temperature = f32::NAN;
        assert!(invalid.validate().is_err());
        invalid.options.temperature = 2.1;
        assert!(invalid.validate().is_err());
        invalid = request();
        invalid.options.max_output_tokens = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn cancellation_is_shared_between_clones() {
        let original = GenerationCancellation::default();
        let worker = original.clone();
        assert!(!worker.is_cancelled());
        original.cancel();
        assert!(worker.is_cancelled());
    }

    struct FakeBackend;

    impl GenerationBackend for FakeBackend {
        fn generate<'a>(
            &'a self,
            request: GenerationRequest,
            cancellation: GenerationCancellation,
            on_event: &'a mut (dyn FnMut(GenerationEvent) + Send),
        ) -> GenerationFuture<'a> {
            Box::pin(async move {
                request.validate()?;
                if cancellation.is_cancelled() {
                    return Err(GenerationError::Cancelled);
                }
                on_event(GenerationEvent::Content("你好".to_owned()));
                on_event(GenerationEvent::Complete);
                Ok(())
            })
        }
    }

    #[test]
    fn cancelled_generation_emits_no_content() {
        let cancellation = GenerationCancellation::default();
        cancellation.cancel();
        let mut events = Vec::new();
        {
            let mut callback = |event| events.push(event);
            let mut future = FakeBackend.generate(request(), cancellation, &mut callback);
            use std::task::{Context, Poll, Wake, Waker};
            struct Noop;
            impl Wake for Noop {
                fn wake(self: std::sync::Arc<Self>) {}
            }
            let waker = Waker::from(std::sync::Arc::new(Noop));
            let mut cx = Context::from_waker(&waker);
            assert_eq!(
                future.as_mut().poll(&mut cx),
                Poll::Ready(Err(GenerationError::Cancelled))
            );
        }
        assert!(events.is_empty());
    }

    #[test]
    fn backend_contract_reports_deltas_and_completion() {
        let mut events = Vec::new();
        {
            let mut callback = |event| events.push(event);
            let mut future =
                FakeBackend.generate(request(), GenerationCancellation::default(), &mut callback);
            use std::task::{Context, Poll, Wake, Waker};
            struct Noop;
            impl Wake for Noop {
                fn wake(self: std::sync::Arc<Self>) {}
            }
            let waker = Waker::from(std::sync::Arc::new(Noop));
            let mut cx = Context::from_waker(&waker);
            assert_eq!(future.as_mut().poll(&mut cx), Poll::Ready(Ok(())));
        }
        assert_eq!(
            events,
            vec![
                GenerationEvent::Content("你好".into()),
                GenerationEvent::Complete
            ]
        );
    }
}
