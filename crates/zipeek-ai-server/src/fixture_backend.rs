use zipeek_ai_core::{
    GenerationBackend, GenerationCancellation, GenerationError, GenerationEvent, GenerationFuture,
    GenerationRequest,
};

pub struct FixtureBackend;

impl GenerationBackend for FixtureBackend {
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
