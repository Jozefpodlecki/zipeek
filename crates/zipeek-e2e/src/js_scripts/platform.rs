pub const BOOTSTRAP_MOUNTED_APP_WAITS_FOR_DICTIONARY_DOWNLOAD_01: &str = r####"
return window.dictionaryWorkers;
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_01: &str = r####"
return {urls: ['*.wasm']};
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_02: &str = r####"
const panel = document.querySelector('.loading-panel');
if (!panel) return false;
const r = panel.getBoundingClientRect();
return Math.abs(r.left + r.width / 2 - innerWidth / 2) < 1 && Math.abs(r.top + r.height / 2 - innerHeight / 2) < 1;
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_03: &str = r####"
const i = document.querySelector('.app-backdrop img');
const root = document.querySelector('#root');
return !!i && !!root && i.complete && i.naturalWidth > 0 && !root.contains(i) &&
    getComputedStyle(i).display !== 'none';
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_04: &str = r####"
return document.documentElement.classList.contains('dark');
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_05: &str = r####"
return !document.documentElement.hasAttribute('data-appearance-ready');
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_06: &str = r####"
return document.documentElement.scrollHeight <= innerHeight;
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_07: &str = r####"
const panel = document.querySelector('.loading-panel');
if (!panel) return false;
const r = panel.getBoundingClientRect();
return r.top >= 0 && r.bottom <= innerHeight && r.left >= 0 && r.right <= innerWidth &&
    Math.abs(r.top + r.height / 2 - innerHeight / 2) < 1;
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_08: &str = r####"
return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_09: &str = r####"
const spinner = document.querySelector('.loading-spinner');
return spinner ? getComputedStyle(spinner).animationName : null;
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_10: &str = r####"
return {urls: []};
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_11: &str = r####"
window.backdropBeforeToggle = document.querySelector('.app-backdrop');
"####;

pub const BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_12: &str = r####"
const root = document.querySelector('#root');
return !!root && !!window.backdropBeforeToggle &&
    window.backdropBeforeToggle === document.querySelector('.app-backdrop') &&
    !root.contains(window.backdropBeforeToggle);
"####;

pub const BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_01: &str = r####"
return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};
"####;

pub const BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_02: &str = r####"
return document.documentElement.classList.contains('dark');
"####;

pub const BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_03: &str = r####"
const done = arguments[arguments.length - 1];
requestAnimationFrame(() => requestAnimationFrame(() => done(true)));
"####;

pub const DATABASE_DICTIONARY_FAILURE_RETRY_AND_REAL_READ_ONLY_QUERIES_01: &str = r####"
return {urls: ['*hanzi*.duckdb*']};
"####;

pub const DATABASE_DICTIONARY_FAILURE_RETRY_AND_REAL_READ_ONLY_QUERIES_02: &str = r####"
return {urls: []};
"####;

pub const DATABASE_ADAPTER_READS_ALL_ROWS_AND_REJECTS_WRITES_01: &str = r####"
return arguments[0].ok;
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_01: &str = r####"
const done = arguments[arguments.length - 1];
function wait() { if (window.releaseDictionary) done(true);
else requestAnimationFrame(wait);
} wait();
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_02: &str = r####"
return window.dictionaryWorkers;
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_03: &str = r####"
return window.dictionaryWorkers;
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_04: &str = r####"
location.hash = '#/practice';
location.hash = '#/browse';
location.hash = '#/';
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_05: &str = r####"
return window.dictionaryDownloads;
"####;

pub const PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_06: &str = r####"
return window.dictionaryWorkers;
"####;

pub const RECENT_ROUTES_CROSSWORD_CLICK_TO_PLACE_AND_RESET_01: &str = r####"
const board = document.querySelector('[data-crossword-board]');
const box = board?.getBoundingClientRect();
return Boolean(box && box.width >= 200 && box.height >= 200 && board.querySelectorAll('[data-crossword-cell]').length >= 4);
"####;

pub const RECENT_ROUTES_DICTIONARY_AUTOCOMPLETE_KEYBOARD_AND_PAGINATION_01: &str = r####"
const pagination = document.querySelector('[data-component="browse-pagination"]');
return !!pagination && getComputedStyle(pagination).justifyContent === 'center';
"####;

pub const SHELL_LANDING_PAGE_LOADS_01: &str = r####"
Storage.prototype.setItem = function() { throw new DOMException('blocked', 'SecurityError');
};
"####;

pub const SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_01: &str = r####"
return getComputedStyle(document.body).color !== getComputedStyle(document.body).backgroundColor;
"####;

pub const SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_02: &str = r####"
return window.innerWidth;
"####;

pub const SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_03: &str = r####"
const drawer = document.querySelector('#app-drawer');
return drawer ? getComputedStyle(drawer).position : null;
"####;

pub const SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_01: &str = r####"
const topbar = document.querySelector('[data-topbar]');
const toggle = document.querySelector('[data-theme-toggle]');
return !!topbar && !!toggle && topbar.contains(toggle);
"####;

pub const SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_02: &str = r####"
const toggle = document.querySelector('[data-theme-toggle]');
const topbar = document.querySelector('[data-topbar]');
if (!toggle || !topbar) return false;
const r = toggle.getBoundingClientRect();
return r.top >= 0 && r.bottom <= topbar.getBoundingClientRect().bottom &&
    innerWidth - r.right <= 24;
"####;

pub const SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_03: &str = r####"
return document.documentElement.scrollHeight <= innerHeight && document.documentElement.scrollWidth <= innerWidth;
"####;

pub const SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_04: &str = r####"
const content = document.querySelector('#main-content');
return !!content && content.getBoundingClientRect().left === 0;
"####;

pub const SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_05: &str = r####"
return document.documentElement.scrollHeight > innerHeight;
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_01: &str = r####"
return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_02: &str = r####"
const done = arguments[arguments.length - 1];
document.fonts.ready.then(() => requestAnimationFrame(() => requestAnimationFrame(() => done(true))));
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_03: &str = r####"
window.scrollTo(0, document.documentElement.scrollHeight);
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_04: &str = r####"
const s = getComputedStyle(document.activeElement);
return document.activeElement.matches('[data-theme-toggle]:focus-visible') && s.outlineStyle !== 'none' &&
    parseFloat(s.outlineWidth) >= 3 && s.outlineOffset === '1px';
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_05: &str = r####"
const s = getComputedStyle(document.querySelector('[data-navigation-tiles] > a'));
return [matchMedia('(prefers-reduced-motion: reduce)').matches, s.translate, s.transform, s.transitionProperty];
"####;

pub const STYLING_SHELL_AND_CARD_STYLING_06: &str = r####"
return matchMedia('(prefers-reduced-motion: reduce)').matches;
"####;

pub const R421_OPEN_SECOND_ZIPEEK_TAB: &str = r####"
window.__zipeekSecondTab = window.open(arguments[0], '_blank');
return !!window.__zipeekSecondTab;
"####;

pub const R421_CLOSE_SECOND_ZIPEEK_TAB: &str = r####"
if (window.__zipeekSecondTab) {
    window.__zipeekSecondTab.close();
    window.__zipeekSecondTab = null;
}
return true;
"####;
