pub const PRACTICE_ANSWER_OPTIONS_PRACTICE_PREFERENCES_AND_CHARACTER_BUILDER_01: &str = r####"
const a=document.querySelector('[aria-label="Available characters"] .character-builder-hanzi');
const b=document.querySelector('[aria-label="Selected characters"] .character-builder-hanzi');
if (!a||!b) return null;
const aa=getComputedStyle(a), bb=getComputedStyle(b);
return {sameFont:aa.fontFamily===bb.fontFamily, sameSize:aa.fontSize===bb.fontSize};
"####;

pub const FIXED_MCQ_SUMMARY_01: &str = r####"
const done=arguments[arguments.length-1];
function wait(){const s=document.querySelector('[data-summary-first-attempt]');
if(s && s.textContent===s.dataset.summaryFirstAttempt)done(true);
else requestAnimationFrame(wait);
} wait();
"####;

pub const FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_01: &str = r####"
window.fixedClock=100000;
Date.now=()=>window.fixedClock;
"####;

pub const FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_02: &str = r####"
window.fixedClock=200000;
document.dispatchEvent(new Event('visibilitychange'));
"####;

pub const FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_03: &str = r####"
window.fixedClock=900000;
window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');
"####;

pub const FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_04: &str = r####"
return window.oldSummaryCanvas?.dataset.summaryChart === 'destroyed';
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_01: &str = r####"
return document.documentElement.classList.contains('dark');
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_02: &str = r####"
const ring = document.querySelector('[data-summary-ring]');
return !!ring && ring.width > 0 && document.documentElement.scrollWidth <= innerWidth;
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_03: &str = r####"
window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');
window.entropy=0;
Object.defineProperty(crypto,'getRandomValues',{configurable:true,value:bytes=>{bytes.fill(0);
bytes[0]=window.entropy++;
return bytes;
}});
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_04: &str = r####"
return window.oldSummaryCanvas?.dataset.summaryChart === 'destroyed';
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_05: &str = r####"
window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');
"####;

pub const FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_06: &str = r####"
return window.oldSummaryCanvas?.dataset.summaryChart === 'destroyed';
"####;

pub const MATCHING_PRESENTATION_01: &str = r####"
document.documentElement.classList.toggle('dark',arguments[0]);
"####;

pub const MATCHING_PRESENTATION_02: &str = r####"
const done=arguments[arguments.length-1];
requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));
"####;

pub const ASSERT_MATCHING_LAYOUT: &str = r####"
const board = document.querySelector('[data-matching]');
if (!board) {
    return false;
}

const boardBounds = board.getBoundingClientRect();
const cards = [...document.querySelectorAll('[data-match-card]')];
if (cards.length === 0 || cards.length % 2 !== 0) {
    return false;
}

const cardsFit = cards.every((card, index) => {
    const partner = cards[index ^ 1];
    if (!partner) {
        return false;
    }

    const bounds = card.getBoundingClientRect();
    const partnerBounds = partner.getBoundingClientRect();
    const listenButton = card.querySelector('[data-listen]');
    const listenBounds = listenButton?.getBoundingClientRect();
    const equalHeight = Math.abs(bounds.height - partnerBounds.height) < 1;
    const audioContained = !listenBounds || (
        listenBounds.bottom <= bounds.bottom &&
        listenBounds.top >= bounds.top
    );

    return equalHeight && audioContained;
});

return boardBounds.width <= 897 &&
    document.documentElement.scrollWidth <= innerWidth &&
    !getComputedStyle(board).backgroundColor.includes('rgba') &&
    cardsFit;
"####;

pub const MATCHING_PRESENTATION_04: &str = r####"
return Math.max(0, document.documentElement.scrollHeight - innerHeight);
"####;

pub const MATCHING_PRESENTATION_05: &str = r####"
const labels = [...document.querySelectorAll('[data-practice-pinyin]')];
return labels.length > 0 && labels.every(label => {
    const listen = label.closest('[data-match-card]')?.querySelector('[data-listen]');
    return !!label.textContent?.trim() && !!listen &&
        label.getBoundingClientRect().right <= listen.getBoundingClientRect().left;
});
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_01: &str = r####"
window.matchNodes=[...document.querySelectorAll('[data-match-side]')];
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_02: &str = r####"
const nodes = [...document.querySelectorAll('[data-match-side]')];
return Array.isArray(window.matchNodes) && nodes.length > 0 &&
    nodes.length === window.matchNodes.length &&
    nodes.every((node, index) => node === window.matchNodes[index]);
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_03: &str = r####"
return document.documentElement.classList.contains('dark');
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_04: &str = r####"
const next = document.querySelector('[data-match-next]');
if (!next) return false;
const bounds = next.getBoundingClientRect();
return document.documentElement.scrollWidth <= innerWidth &&
    bounds.top >= 0 && bounds.bottom <= innerHeight;
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_05: &str = r####"
const label = document.querySelector('[data-practice-pinyin]');
const listen = document.querySelector('[data-listen]');
return !!label && !!listen &&
    label.getBoundingClientRect().bottom <= listen.getBoundingClientRect().top;
"####;

pub const MATCHING_MATCHING_INTERACTION_AND_LAYOUT_06: &str = r####"
document.documentElement.classList.toggle('dark',arguments[0]);
"####;

pub const MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_01: &str = r####"
window.matchClock=100000;
Date.now=()=>window.matchClock;
"####;

pub const MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_02: &str = r####"
window.matchClock=200000;
"####;

pub const MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_03: &str = r####"
window.oldMatchChart=document.querySelector('[data-summary-ring]');
window.matchClock=900000;
"####;

pub const MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_04: &str = r####"
return window.oldMatchChart?.dataset.summaryChart === 'destroyed';
"####;

pub const MATCHING_MATCHING_RECOVERY_SEED_AND_SAME_URL_01: &str = r####"
const cards=[...document.querySelectorAll('[data-match-feedback=correct]')];
return cards.length===8 && cards.every(card=>getComputedStyle(card).animationName==='none' &&
    Number(getComputedStyle(card).opacity)===1 && card.querySelector('[aria-label=Correct]')!==null);
"####;

pub const MATCHING_MATCHING_RECOVERY_SEED_AND_SAME_URL_02: &str = r####"
const post = Worker.prototype.postMessage;
Worker.prototype.postMessage = function(message, ...rest) {
    if (message.type === 'CREATE_PREPARED' &&
        message.data[1].startsWith('SELECT * FROM words WHERE hsk3_level BETWEEN')) {
        message = {
            ...message,
            data: [
                message.data[0],
                message.data[1].replace(' ORDER BY id ASC', ' AND FALSE ORDER BY id ASC')
            ]
        };
    }
    return post.call(this, message, ...rest);
};
location.hash='#/practice/eng-pinyin?offset=8';
"####;

pub const MCQ_ENDLESS_MCQ_ANSWERS_LOCK_AND_CONTINUE_01: &str = r####"
const input = document.createElement('input');
input.id = 'shortcut-test';
document.body.append(input);
input.focus();
"####;

pub const MCQ_ENDLESS_MCQ_ANSWERS_LOCK_AND_CONTINUE_02: &str = r####"
document.getElementById('shortcut-test').remove();
document.querySelector('[data-mcq-prompt]').focus();
"####;

pub const MCQ_ENDLESS_MCQ_SEED_WRAP_AND_RESPONSIVE_LAYOUT_01: &str = r####"
return document.documentElement.classList.contains('dark');
"####;

pub const MIXED_MIXED_SETUP_PROGRESS_SUMMARY_01: &str = r####"
window.mixedClock=100000;
Date.now=()=>window.mixedClock;
Object.defineProperty(crypto,'getRandomValues',{configurable:true,value:bytes=>{bytes.fill(0);
bytes[0]=1;
return bytes;
}});
"####;

pub const MIXED_MIXED_SETUP_PROGRESS_SUMMARY_02: &str = r####"
window.mixedClock=200000;
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_01: &str = r####"
window.practiceQueryMode = 'hold-fail';
location.hash = '#/practice/eng-pinyin?offset=4';
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_02: &str = r####"
const done = arguments[arguments.length - 1];
function wait() { if (window.releasePracticeQuery) done(true);
else requestAnimationFrame(wait);
} wait();
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_03: &str = r####"
location.hash = '#/practice/mcq/meaning-to-hanzi?offset=8';
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_04: &str = r####"
const done = arguments[arguments.length - 1];
window.releasePracticeQuery();
requestAnimationFrame(() => requestAnimationFrame(() => done(true)));
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_05: &str = r####"
window.practiceQueryMode = 'fail';
location.hash = '#/practice/eng-pinyin?offset=12';
"####;

pub const PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_06: &str = r####"
location.hash = '#/practice/mcq/hanzi-to-meaning';
location.hash = '#/practice/hanzi-eng';
location.hash = '#/browse';
"####;

pub const SETUP_SETUP_PROGRESS_MODE_AND_RESPONSIVE_LAYOUT_01: &str = r####"
return [...document.querySelectorAll('[data-practice-exercise][aria-describedby]')].every(button => {
    const descriptionId = button.getAttribute('aria-describedby');
    const target = document.getElementById(descriptionId);
    return target && target.textContent.trim().length > 0;
});
"####;

pub const SETUP_AI_PRACTICE_REQUIRES_MODEL_BEFORE_LAUNCH_01: &str = r####"
const key='zipeek.preferences.v1';
const config=JSON.parse(localStorage.getItem(key)||'{}');
delete config.ai_model;
localStorage.setItem(key,JSON.stringify(config));
"####;

pub const SPEAKING_DENY_MICROPHONE: &str = r#"
const devices = navigator.mediaDevices;
if (!devices || typeof devices.getUserMedia !== 'function') {
    return false;
}
Object.defineProperty(devices, 'getUserMedia', {
    configurable: true,
    value: () => {
        window.__zipeekE2eMicrophoneRequests =
            (window.__zipeekE2eMicrophoneRequests || 0) + 1;
        return Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
    },
});
return true;
"#;

pub const SPEAKING_MICROPHONE_REQUESTS: &str = r#"
    return window.__zipeekE2eMicrophoneRequests || 0;
"#;

pub const SPEAKING_TRACK_STOP_PROBE: &str = r#"
const devices = navigator.mediaDevices;
if (!devices || typeof devices.getUserMedia !== 'function') return false;
const original = devices.getUserMedia.bind(devices);
window.__zipeekE2eTracks = {requests: 0, acquired: 0, stopped: 0, revoked: []};
const originalStop = MediaStreamTrack.prototype.stop;
const stoppedTracks = new WeakSet();
MediaStreamTrack.prototype.stop = function() {
    if (!stoppedTracks.has(this)) {
        stoppedTracks.add(this);
        window.__zipeekE2eTracks.stopped++;
    }
    return originalStop.call(this);
};
const revoke = URL.revokeObjectURL.bind(URL);
URL.revokeObjectURL = url => {
    window.__zipeekE2eTracks.revoked.push(url);
    return revoke(url);
};
Object.defineProperty(devices, 'getUserMedia', {
    configurable: true,
    value: async constraints => {
        window.__zipeekE2eTracks.requests++;
        const stream = await original(constraints);
        for (const track of stream.getTracks()) {
            window.__zipeekE2eTracks.acquired++;
        }
        return stream;
    },
});
return true;
"#;

pub const SPEAKING_TRACK_STOP_COUNTS: &str = r#"
return window.__zipeekE2eTracks || {requests: 0, acquired: 0, stopped: 0};
"#;
