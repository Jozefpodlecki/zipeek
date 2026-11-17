pub const TRANSLATION_CONFIGURE_ENDPOINT_01: &str = r####"
const done = arguments[arguments.length - 1];
const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
for (const [selector, value] of [['[data-ai-base-url]', arguments[0]], ['[data-ai-model-setting]',
    arguments[1]]]) { const input = document.querySelector(selector);
if (!input) { done(false);
return;
} setter.call(input, value);
input.dispatchEvent(new Event('input', {bubbles: true}));
} requestAnimationFrame(() => requestAnimationFrame(() => done(true)));
"####;

pub const TRANSLATION_CONFIGURE_ENDPOINT_02: &str = r####"
const settings = JSON.parse(localStorage.getItem('zipeek.preferences.v1'));
return {url: settings.ai_url, model: settings.ai_model};
"####;

pub const TRANSLATION_OPEN_FROM_PRACTICE_WITH_COUNT_01: &str = r####"
const keys=Object.keys(localStorage).filter(key=>key.startsWith('zipeek.practice.session.v1.'));
return keys.map(key=>JSON.parse(localStorage.getItem(key)));
"####;

pub const TRANSLATION_WAIT_FOR_PRACTICE_PROGRESS_01: &str = r####"
const expected=arguments[0];
const done=arguments[arguments.length-1];
const deadline=Date.now()+20000;
function poll(){const text=document.querySelector('[data-practice-step-progress]')?.textContent??'';
if(text.includes(expected)){done(true);
return;
}if(Date.now()>=deadline){done(false);
return;
}requestAnimationFrame(poll);
}poll();
"####;

pub const TRANSLATION_WAIT_FOR_QUESTION_01: &str = r####"
const expected=arguments[0];
const done=arguments[arguments.length-1];
const deadline = performance.now() + 20000;
function poll() {
    const text = document.querySelector('[data-practice-step-progress]')?.textContent ?? '';
    if (text.includes(expected)) return done(true);
    if (performance.now() >= deadline) return done(false);
    requestAnimationFrame(poll);
}
poll();
"####;

pub const AI_LEARNER_LIVE_LEARNER_01: &str = r####"
const key=Object.keys(localStorage).find(key=>key.startsWith('zipeek.practice.session.v1.'));
const session=JSON.parse(localStorage.getItem(key));
return {current:session.current, outcome:session.steps[0].outcome, direction:session.translation_direction};
"####;

pub const TRANSLATION_FIXTURE_ABANDONED_EVALUATION_DOES_NOT_SCORE_01: &str = r####"
const done = arguments[arguments.length - 1];
setTimeout(() => done(true), 1500);
"####;

pub const TRANSLATION_FIXTURE_PRACTICE_AND_BROWSE_RESPONSIVE_01: &str = r####"
return document.documentElement.scrollWidth > innerWidth + 2
"####;

pub const TRANSLATION_FIXTURE_PRACTICE_AND_BROWSE_RESPONSIVE_02: &str = r####"
return document.documentElement.scrollWidth > innerWidth + 2
"####;

pub const TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_01: &str = r####"
const done = arguments[arguments.length - 1];
const firstBand = document.querySelector('[data-hsk-band="3:1"]');
const sixthBand = document.querySelector('[data-hsk-band="3:6"]');
if (!firstBand || !sixthBand) return done(false);
firstBand.click();
const deadline = performance.now() + 10000;
function after() {
    if (firstBand.getAttribute('aria-pressed') === 'false') {
        sixthBand.click();
        return done(true);
    }
    if (performance.now() >= deadline) return done(false);
    requestAnimationFrame(after);
}
after();
"####;

pub const TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_02: &str = r####"
const key=Object.keys(localStorage).find(key=>key.startsWith('zipeek.practice.session.v1.'));
return JSON.parse(localStorage.getItem(key));
"####;

pub const TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_03: &str = r####"
const key=Object.keys(localStorage).find(key=>key.startsWith('zipeek.practice.session.v1.'));
return JSON.parse(localStorage.getItem(key)).current;
"####;

pub const TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_04: &str = r####"
return Object.keys(localStorage).filter(key=>key.startsWith('zipeek.practice.session.v1.')).length;
"####;

pub const TRANSLATION_FIXTURE_TRANSLATION_RESTORES_SAME_QUESTION_AFTER_NAVIGATION_01: &str = r####"
const key=Object.keys(localStorage).find(key=>key.startsWith('zipeek.practice.session.v1.'));
const value=JSON.parse(localStorage.getItem(key));
return {current:value.current, direction:value.translation_direction, ready:value.steps[0].readiness,
    generated:value.steps[0].generated_instance!==null};
"####;

pub const TRANSLATION_FIXTURE_ENDLESS_TRANSLATION_CAN_FINISH_AND_CLEARS_TEMPORARY_STATE_01: &str = r####"
const key=Object.keys(localStorage).find(key=>key.startsWith('zipeek.practice.session.v1.'));
const value=JSON.parse(localStorage.getItem(key));
return {current:value.current, steps:value.steps.length, endless:value.endless_translation,
    earned:value.steps[0].outcome.scored.earned};
"####;

pub const TRANSLATION_FIXTURE_ENDLESS_TRANSLATION_CAN_FINISH_AND_CLEARS_TEMPORARY_STATE_02: &str = r####"
return Object.keys(localStorage).filter(key=>key.startsWith('zipeek.practice.session.v1.')).length;
"####;

pub const WAIT_FOR_EVALUATION_REQUEST: &str = r####"
const url = arguments[0];
const done = arguments[arguments.length - 1];
const deadline = performance.now() + 15000;
async function waitForEvaluation() {
    if (performance.now() >= deadline) return done(false);
    const response = await fetch(url);
    if (!response.ok) return done(false);
    const payload = await response.json();
    if (!Array.isArray(payload.records)) return done(false);
    if (payload.records.some(record => record.kind === 'evaluation')) return done(true);
    setTimeout(() => {
        waitForEvaluation().catch(() => done(false));
    }, 30);
}
waitForEvaluation().catch(() => done(false));
"####;

pub const READ_LATEST_GENERATION_PROMPT: &str = r####"
const url = arguments[0];
const done = arguments[arguments.length - 1];
fetch(url)
    .then(response => response.json())
    .then(payload => {
        const generations = payload.records.filter(record => record.kind === 'generation');
        const last = generations[generations.length - 1];
        done(last.messages.find(message => message.role === 'user').content);
    })
    .catch(error => done(String(error)));
"####;
