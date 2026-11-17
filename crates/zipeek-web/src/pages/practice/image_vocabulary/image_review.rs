use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct ImageReviewProps {
    pub prompt: String,
    pub on_approve: Callback<MouseEvent>,
    pub image_loaded: bool,
    pub image_failed: bool,
}

#[function_component(ImageReview)]
pub(super) fn image_review(props: &ImageReviewProps) -> Html {
    html! {
        <div
            data-image-review="true"
            class="rounded-xl border border-line p-4 space-y-3"
        >
            <p
                class="text-sm font-semibold"
            >
                {"Review this generated image"}
            </p>
            <p
                class="text-sm text-muted"
            >
                {"Does it clearly show the requested subject(s)? If not, generate another image. The question cannot be answered until you approve the picture."}
            </p>
            <p
                class="text-xs text-muted"
            >
                {&props.prompt}
            </p>
            if props.image_failed {
                <p
                    data-image-review-error="true"
                    role="alert"
                >{"The generated image could not be loaded. Generate another scene."}</p>
            } else if !props.image_loaded {
                <p
                    data-image-review-loading="true"
                    role="status"
                >{"Loading generated image…"}</p>
            }
            <button
                data-image-review-approve="true"
                type="button"
                disabled={!props.image_loaded || props.image_failed}
                onclick={props.on_approve.clone()}
                class="rounded-lg bg-brand-action px-4 py-2 text-white disabled:opacity-50"
            >
                {"Approve image and start"}
            </button>
        </div>
    }
}
