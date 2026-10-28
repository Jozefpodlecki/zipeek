window.e2eDiagnostics = [];
for (const level of ['log', 'warn', 'error']) {
    const original = console[level].bind(console);
    console[level] = (...args) => {
        window.e2eDiagnostics.push({level, text: args.map(String).join(' ')});
        original(...args);
    };
}
addEventListener('error', event => window.e2eDiagnostics.push({level: 'error', text: event.message}));
addEventListener('unhandledrejection', event => window.e2eDiagnostics.push({level: 'rejection', text: String(event.reason)}));
