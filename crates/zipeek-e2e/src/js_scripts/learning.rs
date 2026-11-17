pub const LEARNER_JOURNEYS_RAPID_HSK_SELECTION_REMAINS_RESPONSIVE_01: &str = r####"
for (const id of ['3:2', '3:3', '3:4', '3:5', '3:2', '3:3']) {
    const band = document.querySelector(`[data-hsk-band='${id}']`);
    if (!band) throw new Error(`Missing HSK filter button: ${id}`);
    band.click();
}
"####;

pub const TRANSLATOR_DIAGNOSTICS_STREAMING_PROGRESS_VISIBLE_BEFORE_RESULT_01: &str = r####"
const done = arguments[arguments.length-1];
let tries = 0;
function poll() { const progress = document.querySelector('[data-ai-stream-progress]');
if (progress && Number(progress.dataset.aiStreamChars) >= 128) done(true);
else if (tries++ < 75) setTimeout(poll, 40);
else done(false);
} poll();
"####;

pub const WORD_SETS_PERSONAL_SETS_PERSIST_ACROSS_NAVIGATION_AND_BACKUP_REJECTS_INVALID_DATA_01:
    &str = r####"
const backup = document.querySelector('[data-word-set-backup-json]');
return !!backup && backup.value.includes('E2E saved vocabulary');
"####;

pub const WRITING_TAILWIND_DESKTOP_LAYOUT: &str = r####"
const page = document.querySelector('[data-component="write-hanzi-page"]');
const finder = page?.querySelector('[aria-label="Choose vocabulary for writing"]');
const results = page?.querySelector('[aria-label="Available Chinese words"]');
if (!page || !finder || !results) return false;
const pageStyle = getComputedStyle(page);
return pageStyle.height !== '0px' && pageStyle.overflow === 'hidden' &&
    getComputedStyle(finder).display === 'flex' && getComputedStyle(results).overflowY === 'auto';
"####;

pub const WRITING_TAILWIND_MOBILE_LAYOUT: &str = r####"
const page = document.querySelector('[data-component="write-hanzi-page"]');
const results = page?.querySelector('[aria-label="Available Chinese words"]');
return Boolean(page && results && getComputedStyle(page).overflow !== 'hidden' &&
    getComputedStyle(results).overflowY === 'auto');
"####;

pub const WRITING_VIEWPORT_WIDTH: &str = r####"
return window.innerWidth;
"####;
