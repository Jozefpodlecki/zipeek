use crate::theme::ThemeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundAsset {
    pub path: &'static str,
    pub attribution: Option<&'static str>,
    pub theme: ThemeId,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StaticBackgroundProvider;

impl StaticBackgroundProvider {
    pub async fn load(&self, theme: ThemeId) -> BackgroundAsset {
        BackgroundAsset {
            path: "public/backgrounds/lavender.webp",
            attribution: None,
            theme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct Noop;
    impl Wake for Noop {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn static_provider_does_not_require_a_remote_service() {
        let future = StaticBackgroundProvider.load(ThemeId::Travel);
        let waker = Waker::from(Arc::new(Noop));
        let mut context = Context::from_waker(&waker);
        let asset = match pin!(future).as_mut().poll(&mut context) {
            Poll::Ready(asset) => asset,
            Poll::Pending => panic!("static provider must be ready"),
        };
        assert_eq!(asset.path, "public/backgrounds/lavender.webp");
        assert_eq!(asset.theme, ThemeId::Travel);
    }
}
