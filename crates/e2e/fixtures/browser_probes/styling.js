const selectors = {
    topbar: '[data-topbar]',
    drawer: '#app-drawer',
    main: '#main-content',
    card: '[data-daily-word]',
    tile: '[data-navigation-tiles] > a',
    theme: '[data-theme-toggle]',
    footer: '[data-app-footer]',
};
const elements = Object.fromEntries(Object.entries(selectors).map(([name, selector]) => {
    const element = document.querySelector(selector);
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    return [name, {
        x: rect.x, y: rect.y, width: rect.width, height: rect.height,
        position: style.position, display: style.display, gap: style.gap,
        padding: style.padding, font: style.font, color: style.color,
        background: style.background, border: style.border,
        borderRadius: style.borderRadius, maxWidth: style.maxWidth,
        backdropFilter: style.backdropFilter,
    }];
}));
return JSON.stringify({contrast: window.e2eStyleContrast, width: innerWidth, height: innerHeight, scrollWidth: document.documentElement.scrollWidth, scrollHeight: document.documentElement.scrollHeight, elements}, null, 2);
