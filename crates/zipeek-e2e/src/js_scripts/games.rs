pub const GAME_INTERACTIONS_CLOUD_MATCH_CONFIGURATION_AND_NAVIGATION_01: &str = r####"
document.querySelector('a[aria-label="All games"]')?.scrollIntoView({block:'center', inline:'nearest'});
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_CONFIGURATION_AND_NAVIGATION_02: &str = r####"
const link = document.querySelector('a[aria-label="All games"]');
if (!link) return false;
const rect = link.getBoundingClientRect();
const x = rect.left + rect.width / 2;
const y = rect.top + rect.height / 2;
return y > 0 && y < innerHeight && link.contains(document.elementFromPoint(x, y));
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_01: &str = r####"
const clouds = [...document.querySelectorAll('[data-cloud-pair-id][data-cloud-side]')];
return clouds.length > 0 &&
    clouds.every(cloud => cloud.hasAttribute('data-cloud-motion-progress'));
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_02: &str = r####"
const controller = document.querySelector('[data-cloud-motion-controller]');
if (!controller) return false;
const arena = controller.getBoundingClientRect();
const movingClouds = [...document.querySelectorAll('[data-cloud-side][data-cloud-motion-progress]')];
return movingClouds.length > 0 && movingClouds.every(cloud => {
    const bounds = cloud.getBoundingClientRect();
    const side = cloud.getAttribute('data-cloud-side');
    return side === 'left'
        ? bounds.left < arena.left + bounds.width * 0.35
        : bounds.right > arena.right - bounds.width * 0.35;
});
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_03: &str = r####"
const controller = document.querySelector('[data-cloud-motion-controller]');
if (!controller) return 0;
const arena = controller.getBoundingClientRect();
const clouds = [...document.querySelectorAll('[data-cloud-pair-id][data-cloud-side]')];
return clouds.filter(cloud => {
    const bounds = cloud.getBoundingClientRect();
    return bounds.right > arena.left + 20 &&
        bounds.left < arena.right - 20 &&
        bounds.bottom > arena.top &&
        bounds.top < arena.bottom;
}).length;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_04: &str = r####"
const clouds = [...document.querySelectorAll('[data-cloud-velocity]')];
return clouds.map(cloud => Number(cloud.getAttribute('data-cloud-velocity')));
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_05: &str = r####"
const controller = document.querySelector('[data-cloud-motion-controller]');
if (!controller) return 0;
const arena = controller.getBoundingClientRect();
const clouds = [...document.querySelectorAll('[data-cloud-pair-id][data-cloud-side]')];
return clouds.filter(cloud => {
    const bounds = cloud.getBoundingClientRect();
    return bounds.right > arena.left &&
        bounds.left < arena.right &&
        bounds.bottom > arena.top &&
        bounds.top < arena.bottom;
}).length;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_06: &str = r####"
return document.activeElement?.textContent?.includes('Resume game') ?? false;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_07: &str = r####"
window.__cloudUnpositionedVisible = false;
window.__cloudSpawnObserver = new MutationObserver(() => {
    for (const cloud of document.querySelectorAll('[data-cloud-tile]:not([data-cloud-ghost])')) {
        if (getComputedStyle(cloud).visibility === 'visible' &&
            !cloud.hasAttribute('data-cloud-velocity')) {
            window.__cloudUnpositionedVisible = true;
        }
    }
});
const controller = document.querySelector('[data-cloud-motion-controller]');
if (!controller) {
    window.__cloudSpawnObserver.disconnect();
    throw new Error('Cloud Match motion controller is missing');
}
window.__cloudSpawnObserver.observe(controller, {childList: true, subtree: true});
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_08: &str = r####"
return document.querySelector('[data-cloud-side][data-cloud-matched="false"]')?.getAttribute('data-cloud-wave') ?? '';
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_09: &str = r####"
const controller = document.querySelector('[data-cloud-motion-controller]');
if (!controller) return false;
const arena = controller.getBoundingClientRect();
const clouds = [...document.querySelectorAll('[data-cloud-side][data-cloud-matched="false"]')];
return clouds.length > 0 && clouds.every(cloud => {
    const bounds = cloud.getBoundingClientRect();
    return cloud.dataset.cloudSide === 'left'
        ? bounds.left < arena.left + bounds.width * 0.9
        : bounds.right > arena.right - bounds.width * 0.9;
});
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_10: &str = r####"
if (!window.__cloudSpawnObserver) return false;
window.__cloudSpawnObserver.disconnect();
return window.__cloudUnpositionedVisible === false;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_REPLENISHES_WITHOUT_RESETTING_SURVIVORS_01: &str = r####"
const hud = document.querySelector('[data-component=cloud-match-hud]');
const clouds = [...document.querySelectorAll('[data-cloud-side]:not([data-cloud-ghost])')];
return !!hud && hud.textContent.includes('Continuous') &&
    clouds.length > 0 && clouds.every(cloud => cloud.dataset.cloudWave === '0');
"####;

pub const GAME_INTERACTIONS_TONE_DETECTIVE_SCORES_AND_RESTARTS_01: &str = r####"
sessionStorage.removeItem('zipeek.e2e.speech.events.v1');
"####;

pub const GAME_INTERACTIONS_TONE_DETECTIVE_SCORES_AND_RESTARTS_02: &str = r####"
const events = JSON.parse(sessionStorage.getItem('zipeek.e2e.speech.events.v1') || '[]');
return events.some(x => x.event === 'speak' && x.language === 'zh-CN' && x.text.length > 0);
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_01: &str = r####"
const left = [...document.querySelectorAll('[data-cloud-side=left][data-cloud-pair-id]')].filter(el => !el.disabled);
const right = [...document.querySelectorAll('[data-cloud-side=right][data-cloud-pair-id]')].filter(el => !el.disabled);
const other = right.find(el => el.dataset.cloudPairId !== left[0]?.dataset.cloudPairId);
if (!left[0] || !other) return false;
left[0].click();
other.click();
return true;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_02: &str = r####"
return document.querySelectorAll('[data-cloud-incorrect=true]').length === 2;
"####;

pub const GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_03: &str = r####"
return [...document.querySelectorAll('[data-cloud-incorrect=true]')].length === 0;
"####;

pub const HAS_CLOUD_MOTION_PROGRESS: &str = r####"
return document.querySelector(arguments[0])?.hasAttribute('data-cloud-motion-progress') ?? false;
"####;

pub const REMEMBER_STABLE_CLOUD: &str = r####"
window.__cloudStableNode = document.querySelector(arguments[0]);
"####;

pub const CAPTURE_CLOUD_MOTION_BASELINE: &str = r####"
const cloud = document.querySelector(arguments[0]);
if (!cloud) throw new Error('Cannot capture motion baseline: cloud not found');
window.__cloudMotionBaseline = {
    travel: Number(cloud.getAttribute('data-cloud-motion-progress')),
    timestamp: performance.now(),
    speed: Number(cloud.getAttribute('data-cloud-velocity'))
};
"####;

pub const CLICK_CLOUD_BY_SIDE_AND_PAIR: &str = r####"
const cloud = [...document.querySelectorAll('[data-cloud-side]')].find(cloud =>
    cloud.dataset.cloudSide === arguments[0] &&
    cloud.dataset.cloudPairId === arguments[1]
);
if (!cloud) throw new Error('Cloud with requested side and pair ID not found');
cloud.click();
"####;

pub const PARTNER_CLOUD_HINT_VISIBLE: &str = r####"
const partner = [...document.querySelectorAll('[data-cloud-side]')].find(cloud =>
    cloud.dataset.cloudSide === 'right' &&
    cloud.dataset.cloudPairId === arguments[0]
);
return partner?.dataset.cloudHint === 'true';
"####;

pub const MATCHED_CLOUDS_HIDDEN: &str = r####"
const clouds = [...document.querySelectorAll('[data-cloud-side]')];
const matching = side => clouds.find(cloud =>
    cloud.dataset.cloudSide === side &&
    cloud.dataset.cloudPairId === arguments[0]
);
const left = matching('left');
const right = matching('right');
return !!left && !!right &&
    left.disabled && right.disabled &&
    getComputedStyle(left).visibility === 'hidden' &&
    getComputedStyle(right).visibility === 'hidden';
"####;

pub const SAMPLE_CLOUD_MOTION: &str = r####"
const cloud = document.querySelector(arguments[0]);
const baseline = window.__cloudMotionBaseline;
return [
    baseline.travel,
    Number(cloud.getAttribute('data-cloud-motion-progress')),
    performance.now() - baseline.timestamp,
    baseline.speed
];
"####;

pub const IS_STABLE_CLOUD_NODE: &str = r####"
return window.__cloudStableNode === document.querySelector(arguments[0]);
"####;

pub const CLOUD_MOTION_PROGRESS: &str = r####"
return Number(document.querySelector(arguments[0]).getAttribute('data-cloud-motion-progress'));
"####;

pub const NEW_CLOUD_WAVE_READY: &str = r####"
const cloud = document.querySelector('[data-cloud-side][data-cloud-matched="false"]');
return !!cloud &&
    cloud.getAttribute('data-cloud-wave') !== arguments[0] &&
    cloud.hasAttribute('data-cloud-velocity');
"####;

pub const REMEMBER_ROLLING_SURVIVOR: &str = r####"
window.__rollingSurvivor = document.querySelector(arguments[0]);
"####;

pub const CLICK_MATCHING_CLOUD_PAIR: &str = r####"
const clouds = [...document.querySelectorAll('[data-cloud-side]')];
for (const side of ['left', 'right']) {
    clouds.find(cloud =>
        cloud.dataset.cloudSide === side && cloud.dataset.cloudPairId === arguments[0]
    ).click();
}
"####;

pub const HAS_NEW_REPLACEMENT_CLOUD: &str = r####"
return [...document.querySelectorAll("[data-cloud-side='left']")].some(cloud =>
    !arguments[0].includes(cloud.dataset.cloudPairId) &&
    cloud.dataset.cloudMotionProgress &&
    !cloud.disabled
);
"####;

pub const IS_ROLLING_SURVIVOR_STABLE: &str = r####"
return window.__rollingSurvivor === document.querySelector(arguments[0]);
"####;

pub const CROSSWORD_TAILWIND_BOARD_LAYOUT: &str = r####"
const cell = document.querySelector('[aria-label="Hanzi crossword squares"] button');
if (!cell) return false;
const style = getComputedStyle(cell);
return style.display === 'flex' && style.alignItems === 'center' &&
    style.justifyContent === 'center' && style.overflow === 'hidden';
"####;
