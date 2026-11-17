pub const ARTIFACTS_CAPTURE_BROWSER_DIAGNOSTICS_01: &str = r####"
return JSON.stringify({
    route: location.pathname,
    fragmentRoute: location.hash.split('?')[0],
    console: (window.e2eDiagnostics || []).slice(-100),
    resources: performance.getEntriesByType('resource').map(resource => ({
        type: resource.initiatorType,
        duration: resource.duration,
        bytes: resource.transferSize
    }))
}, null, 2);
"####;

pub const ARTIFACTS_CAPTURE_FAILURE_STATE: &str = r#"
const root = document.querySelector('[data-image-vocabulary], [data-component="speaking-recorder"], [data-exercise="word-mosaics"]');
return JSON.stringify({
    url: location.href,
    component: root?.getAttribute('data-image-vocabulary') !== null && root?.hasAttribute('data-image-vocabulary') ? 'image-vocabulary' :
        root?.getAttribute('data-component') || root?.getAttribute('data-exercise') || null,
    phase: root?.getAttribute('data-recorder-phase') || null,
    text: (root?.innerText || '').slice(0, 1500),
    image: (() => {
        const node = root?.querySelector('img[alt="AI-generated visual vocabulary scene"]');
        return node ? {loaded: node.complete, width: node.naturalWidth} : null;
    })(),
    alerts: [...document.querySelectorAll('[role="alert"]')].map(node => node.innerText.slice(0, 500)),
});
"#;

pub const MOD_PROVE_FAILURE_ISOLATION_01: &str = r####"
localStorage.setItem('e2e-poison', 'true');
document.cookie = 'e2e-poison=true';
"####;

pub const MOD_PROVE_FAILURE_ISOLATION_02: &str = r####"
return !localStorage.getItem('e2e-poison') && !document.cookie.includes('e2e-poison') && !window.e2ePoisoned;
"####;

pub const SCENARIO_WAIT_FOR_DATABASE_01: &str = r####"
return [...document.querySelectorAll('[data-database-state]')].map(node => ({state:node.dataset.databaseState,
    message:node.textContent.slice(0,200)}));
"####;

pub const SCENARIO_WAIT_FOR_DATABASE_02: &str = r####"
return {features: [{name: 'prefers-reduced-motion', value: 'no-preference'}]};
"####;

pub const SCENARIO_WAIT_FOR_DATABASE_03: &str = r####"
return document.getElementById('root')?.getAttribute('data-e2e-speech') ?? null;
"####;

pub const CONFIGURE_IMAGE_FIXTURE: &str = r#"
    const key = 'zipeek.preferences.v1';
    try {
        const settings = JSON.parse(localStorage.getItem(key) || '{}');
        if (!settings || typeof settings !== 'object' || Array.isArray(settings)) {
            return false;
        }
        settings.image_server_url = arguments[0];
        localStorage.setItem(key, JSON.stringify(settings));
        return true;
    } catch (_) {
        return false;
    }
"#;

pub const ASSERT_IMAGE_FIXTURE_HEALTH: &str = r#"
    const url = arguments[0];
    const done = arguments[arguments.length - 1];
    fetch(`${url}/health`)
        .then(response => response.ok ? response.json() : null)
        .then(payload => done(payload?.status === 'ok' && payload.backend === 'placeholder'))
        .catch(() => done(false));
"#;

pub const ASSERT_IMAGE_FIXTURE_GENERATES_IMAGE: &str = r#"
    const url = arguments[0];
    const done = arguments[arguments.length - 1];
    const deadline = Date.now() + 10000;
    async function run() {
        const submission = await fetch(`${url}/v1/images`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
                prompt: 'A blue teapot beside a wooden chair',
                width: 256,
                height: 256
            })
        });
        if (submission.status !== 202) return false;
        const job = await submission.json();
        if (!/^[0-9a-f-]{36}$/.test(job.id)) return false;
        while (Date.now() < deadline) {
            const result = await fetch(`${url}/v1/jobs/${job.id}`);
            if (!result.ok) return false;
            const current = await result.json();
            if (current.id !== job.id || current.status === 'failed') return false;
            if (current.status === 'completed') {
                if (current.image_url !== `/v1/images/${job.id}`) return false;
                const image = await fetch(`${url}${current.image_url}`);
                return image.ok &&
                    image.headers.get('Content-Type')?.startsWith('image/svg+xml') &&
                    (await image.text()).includes('A blue teapot');
            }
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        return false;
    }
    run().then(done).catch(() => done(false));
"#;

pub const CONFIGURE_IMAGE_EXERCISE_FIXTURES: &str = r#"
    const key = 'zipeek.preferences.v1';
    try {
        const settings = JSON.parse(localStorage.getItem(key) || '{}');
        if (!settings || typeof settings !== 'object' || Array.isArray(settings)) {
            return false;
        }
        settings.image_server_url = arguments[0];
        settings.ai_url = arguments[1];
        settings.ai_model = 'fixture-model';
        localStorage.setItem(key, JSON.stringify(settings));
        return true;
    } catch (_) {
        return false;
    }
"#;

pub const ASSERT_AI_FIXTURE_REQUEST_KIND: &str = r#"
    const url = arguments[0];
    const kind = arguments[1];
    const done = arguments[arguments.length - 1];
    fetch(url, {cache: 'no-store'})
        .then(response => response.ok ? response.json() : null)
        .then(data => done(Array.isArray(data?.records) &&
            data.records.some(record => record.kind === kind)))
        .catch(() => done(false));
"#;

pub const FIREFOX_BOOTSTRAP_STORAGE: &str = r####"
localStorage.setItem('zipeek.preferences.v1', arguments[0]);
sessionStorage.setItem('zipeek.e2e.settings.v1', JSON.stringify({speech:'silent'}));
"####;

pub const E2E_BROWSER_USER_AGENT: &str = "return navigator.userAgent;";
pub const E2E_BROWSER_VIEWPORT_WIDTH: &str = "return window.innerWidth;";
