use serde::Deserialize;

#[derive(Deserialize)]
struct JobResponse {
    id: String,
    status: String,
    image_url: Option<String>,
    error: Option<String>,
}

pub(crate) async fn generate(api_url: &str, prompt: &str) -> Result<String, String> {
    let base = super::validate_api_url(api_url).map_err(str::to_owned)?;
    let response = gloo::net::http::Request::post(&format!("{base}/v1/images"))
        .json(&serde_json::json!({ "prompt": prompt, "width": 512, "height": 512 }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Image server request failed: {e}"))?;
    if !response.ok() {
        return Err(format!("Image server returned HTTP {}", response.status()));
    }
    let submitted: JobResponse = response.json().await.map_err(|e| e.to_string())?;
    if !is_uuid(&submitted.id) {
        return Err("Image server returned an invalid job identifier".into());
    }
    if !matches!(
        submitted.status.as_str(),
        "queued" | "running" | "completed"
    ) {
        return Err("Image server rejected the submitted image job".into());
    }
    for _ in 0..90 {
        let (sender, receiver) = futures::channel::oneshot::channel();
        let timer = gloo::timers::callback::Timeout::new(2000, move || {
            let _ = sender.send(());
        });
        let _ = receiver.await;
        drop(timer);
        let response = gloo::net::http::Request::get(&format!("{base}/v1/jobs/{}", submitted.id))
            .send()
            .await
            .map_err(|e| format!("Could not poll image job: {e}"))?;
        if !response.ok() {
            return Err(format!("Image job returned HTTP {}", response.status()));
        }
        let job: JobResponse = response.json().await.map_err(|e| e.to_string())?;
        if job.id != submitted.id {
            return Err("Image server returned a different job".into());
        }
        match job.status.as_str() {
            "completed" => {
                let expected = format!("/v1/images/{}", submitted.id);
                if job.image_url.as_deref() != Some(expected.as_str()) {
                    return Err("Image server returned an unexpected image path".into());
                }
                return Ok(format!("{base}{expected}"));
            }
            "failed" => {
                return Err(job
                    .error
                    .map(|reason| {
                        reason
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(300)
                            .collect::<String>()
                    })
                    .filter(|reason| !reason.trim().is_empty())
                    .unwrap_or_else(|| "Image generation failed".into()));
            }
            "queued" | "running" => (),
            _ => return Err("Image server returned an unknown job status".into()),
        }
    }
    Err("Image generation timed out; the server job may still be running".into())
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.chars().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}
