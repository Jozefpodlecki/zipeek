use gloo::timers::callback::Timeout;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub(crate) enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ToastMessage {
    pub text: String,
    pub kind: ToastKind,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ToastBus(pub Callback<ToastMessage>);

impl ToastBus {
    pub fn success(&self, text: impl Into<String>) {
        self.0.emit(ToastMessage {
            text: text.into(),
            kind: ToastKind::Success,
        });
    }
    pub fn error(&self, text: impl Into<String>) {
        self.0.emit(ToastMessage {
            text: text.into(),
            kind: ToastKind::Error,
        });
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct ToastProviderProps {
    pub children: Children,
}

#[function_component(ToastProvider)]
pub(crate) fn toast_provider(props: &ToastProviderProps) -> Html {
    let toast = use_state(|| None::<(u64, ToastMessage)>);
    let sequence = use_mut_ref(|| 0_u64);
    let publish = {
        let toast = toast.clone();
        let sequence = sequence.clone();
        Callback::from(move |message: ToastMessage| {
            let mut current = sequence.borrow_mut();
            *current = current.wrapping_add(1);
            toast.set(Some((*current, message)));
        })
    };
    {
        let publish = publish.clone();
        use_effect_with((), move |_| {
            let listeners = web_sys::window().map(|window| {
                let fallback = publish.clone();
                let unavailable = publish.clone();
                (
                    gloo::events::EventListener::new(&window, "zipeek:speech-browser-fallback", move |_| {
                        fallback.emit(ToastMessage { text: "Native pronunciation unavailable. Using a Mandarin browser voice.".to_owned(), kind: ToastKind::Success });
                    }),
                    gloo::events::EventListener::new(&window, "zipeek:speech-unavailable", move |_| {
                        unavailable.emit(ToastMessage { text: "Mandarin audio is unavailable. No suitable browser voice was found.".to_owned(), kind: ToastKind::Error });
                    }),
                )
            });
            move || drop(listeners)
        });
    }
    {
        let toast = toast.clone();
        use_effect_with((*toast).clone(), move |current| {
            let timer = current.as_ref().map(|(sequence, _)| {
                let toast = toast.clone();
                let sequence = *sequence;
                Timeout::new(3800, move || {
                    if (*toast)
                        .as_ref()
                        .is_some_and(|(active, _)| *active == sequence)
                    {
                        toast.set(None);
                    }
                })
            });
            move || drop(timer)
        });
    }
    let dismiss = {
        let toast = toast.clone();
        Callback::from(move |_| toast.set(None))
    };
    let toast_content = {
        if let Some((_, message)) = (*toast).as_ref() {
            let element_classes_1 = classes!(
                "fixed",
                "bottom-5",
                "left-5",
                "z-[120]",
                "flex",
                "max-w-sm",
                "items-center",
                "gap-3",
                "rounded-xl",
                "border",
                "bg-surface",
                "px-4",
                "py-3",
                "text-sm",
                "shadow-xl",
                if message.kind == ToastKind::Error {
                    "border-red-500/60"
                } else {
                    "border-emerald-500/60"
                },
            );
            html! {
                <div
                    data-component="app-toast"
                    role={if message.kind == ToastKind::Error {"alert"} else {"status"}}
                    class={element_classes_1}
                >
                    <span
                        aria-hidden="true"
                        class="font-bold text-accent"
                    >
                        {if message.kind == ToastKind::Error {"!"} else {"✓"}}
                    </span>
                    <span
                        class="flex-1"
                    >
                        {&message.text}
                    </span>
                    <button
                        type="button"
                        aria-label="Dismiss notification"
                        onclick={dismiss}
                        class="rounded px-2 py-1 text-muted hover:bg-raised"
                    >
                        {"×"}
                    </button>
                </div>
            }
        } else {
            Html::default()
        }
    };
    html! {
        <ContextProvider<ToastBus> context={ToastBus(publish)}>
            {props.children.clone()}
            {toast_content}
        </ContextProvider<ToastBus>>
    }
}
