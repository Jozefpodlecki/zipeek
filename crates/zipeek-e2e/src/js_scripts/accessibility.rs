pub const ACCESSIBILITY_MATRIX_VISIT_AND_CHECK_01: &str = r####"
const ids = new Set(); const repeated = [];
           for (const element of document.querySelectorAll('[id]')) {
             if (ids.has(element.id)) repeated.push(element.id);
             ids.add(element.id);
           }
           const unnamed = [...document.querySelectorAll('main button, [data-component="main-content"] button')]
             .filter(el => el.getClientRects().length && !el.disabled)
             .filter(el => !(el.getAttribute('aria-label') || el.getAttribute('aria-labelledby') ||
               el.getAttribute('title') || el.textContent.trim()))
             .map(el => el.outerHTML.slice(0, 180));
           return {repeated, unnamed};
"####;

pub const RESPONSIVE_MATRIX_CHECK_LAYOUT_01: &str = r####"
return {viewport: innerWidth, document: document.documentElement.scrollWidth, body: document.body.scrollWidth};
"####;
