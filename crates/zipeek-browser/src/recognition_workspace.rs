use std::{cell::RefCell, fmt, rc::Rc};

use crate::recognition_state::CanvasPoint;

use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    CanvasRenderingContext2d, ClipboardEvent, Document, Event, EventTarget, File,
    HtmlCanvasElement, HtmlImageElement, HtmlInputElement, PointerEvent, Url, Window,
};

#[wasm_bindgen(inline_js = r#"
    export function observeRecognitionTheme(root, browserWindow, callback) {
        const observer = new MutationObserver(callback);
        observer.observe(root, {
            attributes: true,
            attributeFilter: ['class', 'data-theme', 'data-theme-preset', 'style']
        });
        const scheme = browserWindow.matchMedia?.('(prefers-color-scheme: dark)');
        scheme?.addEventListener?.('change', callback);
        return () => {
            observer.disconnect();
            scheme?.removeEventListener?.('change', callback);
        };
    }
"#)]
unsafe extern "C" {
    #[wasm_bindgen(js_name = observeRecognitionTheme)]
    #[wasm_bindgen(catch)]
    fn observe_recognition_theme(
        root: &web_sys::Element,
        window: &Window,
        callback: &Function,
    ) -> Result<Function, JsValue>;
}

#[derive(Debug)]
pub enum RecognitionWorkspaceError {
    CanvasContextUnavailable,
    ThemeUnavailable,
    DocumentRootUnavailable,
    EmptyInput,
    UnsupportedImage,
    InvalidImageDimensions,
    BrowserApi {
        operation: &'static str,
        message: String,
    },
}

impl RecognitionWorkspaceError {
    fn browser(operation: &'static str, error: JsValue) -> Self {
        let message = error.as_string().unwrap_or_else(|| {
            js_sys::Reflect::get(&error, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
                .unwrap_or_else(|| "browser API rejected the operation".to_owned())
        });
        Self::BrowserApi { operation, message }
    }
}

impl fmt::Display for RecognitionWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanvasContextUnavailable => write!(formatter, "Canvas 2D context is unavailable"),
            Self::ThemeUnavailable => write!(formatter, "Canvas theme is unavailable"),
            Self::DocumentRootUnavailable => write!(formatter, "Document root is unavailable"),
            Self::EmptyInput => write!(formatter, "Add a drawing or image first"),
            Self::UnsupportedImage => write!(
                formatter,
                "Use a nonempty PNG, JPEG or WebP image smaller than 5 MB"
            ),
            Self::InvalidImageDimensions => write!(
                formatter,
                "Image dimensions must be between 1 and 4096 pixels"
            ),
            Self::BrowserApi { operation, message } => {
                write!(formatter, "{operation} failed: {message}")
            }
        }
    }
}

impl std::error::Error for RecognitionWorkspaceError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecognitionInputKind {
    Empty,
    Drawing,
    Image,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecognitionInputState {
    pub kind: RecognitionInputKind,
    pub can_undo: bool,
}

#[derive(Debug)]
pub enum RecognitionWorkspaceEvent {
    InputChanged(RecognitionInputState),
    Error(RecognitionWorkspaceError),
}

pub type RecognitionWorkspaceCallback = Rc<dyn Fn(RecognitionWorkspaceEvent)>;

struct Drawing {
    canvas: HtmlCanvasElement,
    window: Window,
    document: Document,
    image: Option<HtmlImageElement>,
    strokes: Vec<Vec<CanvasPoint>>,
    pointer: Option<i32>,
    revision: u64,
    active: bool,
    on_event: RecognitionWorkspaceCallback,
}

impl Drawing {
    fn input_state(&self) -> RecognitionInputState {
        use RecognitionInputKind::{Drawing, Empty, Image, Mixed};
        let kind = match (self.image.is_some(), self.strokes.is_empty()) {
            (false, true) => Empty,
            (false, false) => Drawing,
            (true, true) => Image,
            (true, false) => Mixed,
        };
        RecognitionInputState {
            kind,
            can_undo: !self.strokes.is_empty(),
        }
    }

    fn notify(&self) {
        (self.on_event)(RecognitionWorkspaceEvent::InputChanged(self.input_state()));
    }

    fn error(&self, error: RecognitionWorkspaceError) {
        (self.on_event)(RecognitionWorkspaceEvent::Error(error));
    }

    fn repaint(&self) -> Result<(), RecognitionWorkspaceError> {
        let context = self
            .canvas
            .get_context("2d")
            .map_err(|error| RecognitionWorkspaceError::browser("Get canvas context", error))?
            .ok_or(RecognitionWorkspaceError::CanvasContextUnavailable)?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|error| {
                RecognitionWorkspaceError::browser("Convert canvas context", error.into())
            })?;
        let width = f64::from(self.canvas.width());
        let height = f64::from(self.canvas.height());
        let style = self
            .window
            .get_computed_style(&self.canvas)
            .map_err(|error| RecognitionWorkspaceError::browser("Read canvas theme", error))?
            .ok_or(RecognitionWorkspaceError::ThemeUnavailable)?;
        context.clear_rect(0.0, 0.0, width, height);
        let background = style
            .get_property_value("background-color")
            .map_err(|error| RecognitionWorkspaceError::browser("Read background color", error))?;
        context.set_fill_style_str(&background);
        context.fill_rect(0.0, 0.0, width, height);
        if let Some(image) = &self.image {
            let image_width = f64::from(image.natural_width());
            let image_height = f64::from(image.natural_height());
            if image_width > 0.0 && image_height > 0.0 {
                let scale = (width / image_width).min(height / image_height);
                let scaled_width = image_width * scale;
                let scaled_height = image_height * scale;
                context
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        image,
                        (width - scaled_width) / 2.0,
                        (height - scaled_height) / 2.0,
                        scaled_width,
                        scaled_height,
                    )
                    .map_err(|error| {
                        RecognitionWorkspaceError::browser("Draw uploaded image", error)
                    })?;
            }
        }
        let stroke_color = style
            .get_property_value("color")
            .map_err(|error| RecognitionWorkspaceError::browser("Read stroke color", error))?;
        context.set_stroke_style_str(&stroke_color);
        context.set_line_width(7.0);
        context.set_line_cap("round");
        context.set_line_join("round");
        for stroke in &self.strokes {
            if let Some(first) = stroke.first() {
                context.begin_path();
                context.move_to(first.x, first.y);
                for next in stroke.iter().skip(1) {
                    context.line_to(next.x, next.y);
                }
                if stroke.len() == 1 {
                    context.line_to(first.x + 0.1, first.y + 0.1);
                }
                context.stroke();
            }
        }
        Ok(())
    }

    fn point(&self, event: &PointerEvent) -> Option<CanvasPoint> {
        let rect = self.canvas.get_bounding_client_rect();
        crate::recognition_state::canvas_point(
            f64::from(event.client_x()),
            f64::from(event.client_y()),
            rect.left(),
            rect.top(),
            rect.width(),
            rect.height(),
            f64::from(self.canvas.width()),
            f64::from(self.canvas.height()),
        )
    }
}

struct Listener {
    target: EventTarget,
    name: &'static str,
    callback: Closure<dyn FnMut(Event)>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self
            .target
            .remove_event_listener_with_callback(self.name, self.callback.as_ref().unchecked_ref());
    }
}

pub struct RecognitionWorkspace {
    drawing: Rc<RefCell<Drawing>>,
    listeners: Vec<Listener>,
    theme_callback: Closure<dyn FnMut()>,
    theme_cleanup: Function,
}

fn listen(
    listeners: &mut Vec<Listener>,
    target: &EventTarget,
    name: &'static str,
    action: impl FnMut(Event) + 'static,
) -> Result<(), RecognitionWorkspaceError> {
    let callback = Closure::wrap(Box::new(action) as Box<dyn FnMut(Event)>);
    target
        .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
        .map_err(|error| RecognitionWorkspaceError::browser("Register event listener", error))?;
    listeners.push(Listener {
        target: target.clone(),
        name,
        callback,
    });
    Ok(())
}

async fn install_image(
    drawing: Rc<RefCell<Drawing>>,
    file: File,
) -> Result<(), RecognitionWorkspaceError> {
    let kind = file.type_();
    if !file.size().is_finite()
        || !crate::recognition_state::allowed_image(&kind, file.size() as u64)
    {
        return Err(RecognitionWorkspaceError::UnsupportedImage);
    }
    {
        let current = drawing.borrow();
        if (current.image.is_some() || !current.strokes.is_empty())
            && !current
                .window
                .confirm_with_message("Replace the current image and drawing?")
                .map_err(|error| {
                    RecognitionWorkspaceError::browser("Confirm image replacement", error)
                })?
        {
            return Ok(());
        }
    }
    let revision = {
        let mut state = drawing.borrow_mut();
        state.revision = state.revision.wrapping_add(1);
        state.revision
    };
    let url = Url::create_object_url_with_blob(&file)
        .map_err(|error| RecognitionWorkspaceError::browser("Create image URL", error))?;
    let decoded = async {
        let image = drawing
            .borrow()
            .document
            .create_element("img")
            .map_err(|error| RecognitionWorkspaceError::browser("Create image element", error))?
            .dyn_into::<HtmlImageElement>()
            .map_err(|error| {
                RecognitionWorkspaceError::browser("Convert image element", error.into())
            })?;
        image.set_src(&url);
        JsFuture::from(image.decode())
            .await
            .map_err(|error| RecognitionWorkspaceError::browser("Decode image", error))?;
        if image.natural_width() == 0
            || image.natural_height() == 0
            || image.natural_width() > 4096
            || image.natural_height() > 4096
        {
            return Err(RecognitionWorkspaceError::InvalidImageDimensions);
        }
        Ok(image)
    }
    .await;
    let _ = Url::revoke_object_url(&url);
    let image = decoded?;
    let mut state = drawing.borrow_mut();
    if !state.active || state.revision != revision {
        return Ok(());
    }
    state.image = Some(image);
    state.strokes.clear();
    state.repaint()?;
    state.notify();
    Ok(())
}

fn spawn_upload(drawing: Rc<RefCell<Drawing>>, file: File) {
    let started_revision = drawing.borrow().revision;
    spawn_local(async move {
        if let Err(error) = install_image(drawing.clone(), file).await {
            let state = drawing.borrow();
            if state.active
                && (state.revision == started_revision
                    || state.revision == started_revision.wrapping_add(1))
            {
                state.error(error);
            }
        }
    });
}

impl RecognitionWorkspace {
    pub fn mount(
        canvas: &HtmlCanvasElement,
        input: &HtmlInputElement,
        window: &Window,
        document: &Document,
        on_event: RecognitionWorkspaceCallback,
    ) -> Result<Self, RecognitionWorkspaceError> {
        let drawing = Rc::new(RefCell::new(Drawing {
            canvas: canvas.clone(),
            window: window.clone(),
            document: document.clone(),
            image: None,
            strokes: Vec::new(),
            pointer: None,
            revision: 0,
            active: true,
            on_event,
        }));
        drawing.borrow().repaint()?;
        drawing.borrow().notify();
        let mut listeners = Vec::new();
        let target: EventTarget = canvas.clone().unchecked_into();
        let input_target: EventTarget = input.clone().unchecked_into();
        let window_target: EventTarget = window.clone().unchecked_into();

        let down_state = drawing.clone();
        listen(&mut listeners, &target, "pointerdown", move |event| {
            let Some(pointer) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            if pointer.pointer_type() == "mouse" && pointer.button() != 0 {
                return;
            }
            let mut state = down_state.borrow_mut();
            if state.pointer.is_some() {
                return;
            }
            if let Some(point) = state.point(pointer) {
                state.pointer = Some(pointer.pointer_id());
                state.revision = state.revision.wrapping_add(1);
                state.strokes.push(vec![point]);
                let _ = state.canvas.focus();
                let _ = state.canvas.set_pointer_capture(pointer.pointer_id());
                if let Err(error) = state.repaint() {
                    state.error(error);
                }
                state.notify();
            }
        })?;

        let move_state = drawing.clone();
        listen(&mut listeners, &target, "pointermove", move |event| {
            let Some(pointer) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            let mut state = move_state.borrow_mut();
            if state.pointer != Some(pointer.pointer_id()) {
                return;
            }
            if let Some(point) = state.point(pointer) {
                if let Some(stroke) = state.strokes.last_mut() {
                    stroke.push(point);
                }
                if let Err(error) = state.repaint() {
                    state.error(error);
                }
            }
        })?;

        for name in ["pointerup", "pointercancel"] {
            let up_state = drawing.clone();
            listen(&mut listeners, &target, name, move |event| {
                let Some(pointer) = event.dyn_ref::<PointerEvent>() else {
                    return;
                };
                let mut state = up_state.borrow_mut();
                if state.pointer == Some(pointer.pointer_id()) {
                    state.pointer = None;
                }
            })?;
        }

        let upload_state = drawing.clone();
        let file_input = input.clone();
        listen(&mut listeners, &input_target, "change", move |_| {
            let file = file_input.files().and_then(|files| files.item(0));
            file_input.set_value("");
            if let Some(file) = file {
                spawn_upload(upload_state.clone(), file);
            }
        })?;

        let paste_state = drawing.clone();
        let paste_document = document.clone();
        listen(&mut listeners, &window_target, "paste", move |event| {
            let Some(clipboard) = event.dyn_ref::<ClipboardEvent>() else {
                return;
            };
            let focused = paste_document
                .active_element()
                .and_then(|element| {
                    element
                        .closest("[data-component='recognize-hanzi-page']")
                        .ok()
                        .flatten()
                })
                .is_some();
            if !focused {
                return;
            }
            let Some(files) = clipboard.clipboard_data().and_then(|data| data.files()) else {
                return;
            };
            let file = (0..files.length())
                .filter_map(|index| files.item(index))
                .find(|file| file.type_().starts_with("image/"));
            if let Some(file) = file {
                clipboard.prevent_default();
                spawn_upload(paste_state.clone(), file);
            }
        })?;

        let theme_state = drawing.clone();
        let theme_callback = Closure::wrap(Box::new(move || {
            let state = theme_state.borrow();
            if state.active {
                if let Err(error) = state.repaint() {
                    state.error(error);
                }
            }
        }) as Box<dyn FnMut()>);
        let root = document
            .document_element()
            .ok_or(RecognitionWorkspaceError::DocumentRootUnavailable)?;
        let theme_cleanup =
            observe_recognition_theme(&root, window, theme_callback.as_ref().unchecked_ref())
                .map_err(|error| RecognitionWorkspaceError::browser("Observe theme", error))?;
        Ok(Self {
            drawing,
            listeners,
            theme_callback,
            theme_cleanup,
        })
    }

    pub fn capture(&self) -> Result<String, RecognitionWorkspaceError> {
        let state = self.drawing.borrow();
        if state.image.is_none() && state.strokes.is_empty() {
            return Err(RecognitionWorkspaceError::EmptyInput);
        }
        state
            .canvas
            .to_data_url_with_type("image/png")
            .map_err(|error| RecognitionWorkspaceError::browser("Capture canvas image", error))
    }

    pub fn undo(&self) {
        let mut state = self.drawing.borrow_mut();
        if !state.strokes.is_empty() {
            state.revision = state.revision.wrapping_add(1);
            state.strokes.pop();
            if let Err(error) = state.repaint() {
                state.error(error);
            }
            state.notify();
        }
    }

    pub fn clear(&self) {
        let mut state = self.drawing.borrow_mut();
        state.revision = state.revision.wrapping_add(1);
        state.pointer = None;
        state.strokes.clear();
        state.image = None;
        if let Err(error) = state.repaint() {
            state.error(error);
        }
        state.notify();
    }
}

impl Drop for RecognitionWorkspace {
    fn drop(&mut self) {
        {
            let mut state = self.drawing.borrow_mut();
            state.active = false;
            state.revision = state.revision.wrapping_add(1);
            state.image = None;
            state.strokes.clear();
        }
        let _ = self.theme_cleanup.call0(&JsValue::UNDEFINED);
        self.listeners.clear();
    }
}
