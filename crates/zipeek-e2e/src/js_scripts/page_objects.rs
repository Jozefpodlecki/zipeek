pub const BROWSE_SET_SEARCH_QUERY_01: &str = r####"
const input = document.querySelector('[data-browse-query]');
if (!input) return false;
const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
setter.call(input, arguments[0]);
input.dispatchEvent(new Event('input', { bubbles: true }));
return true;
"####;

pub const BROWSE_SEARCH_SNAPSHOT_01: &str = r####"
const input = document.querySelector('[data-browse-query]');
const cards = Array.from(document.querySelectorAll('[data-component="browse-result"]'));
return {
    query: input?.value ?? null,
    headwords: cards.map(card =>
        card.querySelector('[data-browse-headword]')?.textContent?.trim() ?? ''
    ),
    ids: cards.map(card => card.getAttribute('data-browse-word-id')),
    loading: !!document.querySelector('[data-browse-loading]'),
    error: document.querySelector('[data-browse-error]')?.textContent?.trim() ?? null,
    empty: document.querySelector('[data-browse-empty]')?.textContent?.trim() ?? null
};
"####;

pub const BROWSER_SET_LOCAL_STORAGE_01: &str = r####"
localStorage.setItem(arguments[0], arguments[1]);
"####;

pub const BROWSER_REMOVE_LOCAL_STORAGE_01: &str = r####"
localStorage.removeItem(arguments[0]);
"####;

pub const BROWSER_LOCAL_STORAGE_01: &str = r####"
return localStorage.getItem(arguments[0]);
"####;

pub const BROWSER_NO_HORIZONTAL_OVERFLOW_01: &str = r####"
return document.documentElement.scrollWidth <= innerWidth;
"####;

pub const BROWSER_NAVIGATE_HASH_01: &str = r####"
location.hash = arguments[0];
"####;

pub const BROWSER_ELEMENT_INSIDE_VIEWPORT_01: &str = r####"
const element = document.querySelector(arguments[0]);
if (!element) return false;
const bounds = element.getBoundingClientRect();
return bounds.top >= 0 && bounds.bottom <= innerHeight && bounds.left >= 0 && bounds.right <= innerWidth;
"####;

pub const BROWSER_HORIZONTAL_OVERFLOW_PIXELS_01: &str = r####"
return Math.max(0, document.documentElement.scrollWidth - innerWidth);
"####;

pub const BROWSER_SET_PREFERENCE_01: &str = r####"
const key = 'zipeek.preferences.v1';
const settings = JSON.parse(localStorage.getItem(key) || '{}');
settings[arguments[0]] = arguments[1];
localStorage.setItem(key, JSON.stringify(settings));
"####;

pub const BROWSER_PREFERENCE_01: &str = r####"
const settings = JSON.parse(localStorage.getItem('zipeek.preferences.v1') || '{}');
return settings[arguments[0]] ?? null;
"####;

pub const BROWSER_WAIT_FOR_HASH_ROUTE_01: &str = r####"
return location.hash
"####;

pub const NAVIGATION_CURRENT_HASH_01: &str = r####"
return location.hash
"####;

pub const NAVIGATION_SKIP_LINK_TARGET_EXISTS_01: &str = r####"
const link = document.querySelector('[data-skip-link]');
if (!link) return false;
const href = link.getAttribute('href');
return Boolean(href && href.startsWith('#') && document.getElementById(href.slice(1)) !== null);
"####;

pub const NAVIGATION_HAS_HORIZONTAL_OVERFLOW_01: &str = r####"
return document.documentElement.scrollWidth > innerWidth + 1;
"####;
