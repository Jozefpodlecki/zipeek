use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ApiError,
    jobs::{ImageJob, JobCoordinator},
};

#[derive(Serialize)]
struct Health {
    status: &'static str,
    backend: &'static str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateRequest {
    prompt: String,
    width: u32,
    height: u32,
}

pub fn router(coordinator: Arc<JobCoordinator>, allowed_origins: Vec<String>) -> Router {
    let allowed_origins = Arc::new(allowed_origins);
    Router::new()
        .route("/health", get(health))
        .route("/v1/images", post(generate))
        .route("/v1/jobs/{id}", get(job))
        .route("/v1/images/{id}", get(image))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(
            move |request: Request<axum::body::Body>, next: Next| {
                let allowed_origins = Arc::clone(&allowed_origins);
                async move { local_browser_cors(request, next, &allowed_origins).await }
            },
        ))
        .with_state(coordinator)
}

async fn health(State(coordinator): State<Arc<JobCoordinator>>) -> Json<Health> {
    Json(Health {
        status: "ok",
        backend: coordinator.backend_name(),
    })
}

async fn generate(
    State(coordinator): State<Arc<JobCoordinator>>,
    Json(request): Json<GenerateRequest>,
) -> Result<(StatusCode, Json<ImageJob>), ApiError> {
    if request.prompt.trim().is_empty()
        || request.prompt.chars().count() > 500
        || !matches!(request.width, 256 | 512 | 768)
        || !matches!(request.height, 256 | 512 | 768)
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "invalid prompt or dimensions",
        ));
    }
    let job = coordinator
        .submit(request.prompt, request.width, request.height)
        .ok_or(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "image job queue is full",
        ))?;
    Ok((StatusCode::ACCEPTED, Json(job)))
}

async fn job(
    State(coordinator): State<Arc<JobCoordinator>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ImageJob>, ApiError> {
    coordinator
        .get_job(id)
        .map(Json)
        .ok_or(ApiError(StatusCode::NOT_FOUND, "job not found"))
}

async fn image(
    State(coordinator): State<Arc<JobCoordinator>>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let (metadata, bytes) = coordinator
        .get_image(id)
        .await
        .map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "storage read failed"))?
        .ok_or(ApiError(StatusCode::NOT_FOUND, "image not found"))?;
    let content_type = HeaderValue::from_str(&metadata.media_type)
        .map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "invalid image metadata"))?;
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=3600"),
            ),
            (
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn local_browser_cors(
    request: Request<axum::body::Body>,
    next: Next,
    allowed_origins: &[String],
) -> Response {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|header| header.to_str().ok())
        .filter(|value| {
            crate::is_default_dev_origin(value)
                || allowed_origins
                    .iter()
                    .any(|allowed| allowed.as_str() == *value)
        })
        .map(str::to_owned);
    if request.method() == Method::OPTIONS && origin.is_some() {
        let mut response = StatusCode::NO_CONTENT.into_response();
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type"),
        );
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
        if let Some(value) = origin.and_then(|value| HeaderValue::from_str(&value).ok()) {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        }
        return response;
    }
    let mut response = next.run(request).await;
    if let Some(value) = origin.and_then(|value| HeaderValue::from_str(&value).ok()) {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    response
}
