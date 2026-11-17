pub const RESET_SPEECH_EVENTS: &str = "sessionStorage.removeItem('zipeek.e2e.speech.events.v1');";

pub const ASSERT_CHINESE_SPEECH: &str = r#"
    const events = JSON.parse(
        sessionStorage.getItem('zipeek.e2e.speech.events.v1') || '[]'
    ).filter(event => event.event === 'speak');
    return events.length > 0 && events.every(event =>
        event.text.length > 0 &&
        event.language === 'zh-CN' &&
        /[\u3400-\u9fff]/.test(event.text)
    );
"#;

pub const ASSERT_MATCHING_LISTEN: &str = r#"
    const events = JSON.parse(
        sessionStorage.getItem('zipeek.e2e.speech.events.v1') || '[]'
    ).filter(event => event.event === 'speak');
    return events.length === 1 &&
        events[0].language === 'zh-CN' &&
        /[\u3400-\u9fff]/.test(events[0].text) &&
        !document.querySelector('[data-match-selected=true]');
"#;

pub const ASSERT_MATCHING_CARDS_DISABLED: &str = r#"
    const cards = [...document.querySelectorAll('[data-match-side]')];
    return cards.length > 0 && cards.every(card => card.disabled);
"#;

pub const WAIT_FOR_MATCHING_FEEDBACK_TO_CLEAR: &str = r#"
    const done = arguments[arguments.length - 1];
    function waitForFeedback() {
        if (!document.querySelector('[data-match-feedback="wrong"]')) {
            done(true);
            return;
        }
        requestAnimationFrame(waitForFeedback);
    }
    waitForFeedback();
"#;

pub const WAIT_FOR_MATCHING_SUMMARY_TO_SETTLE: &str = r#"
    const done = arguments[arguments.length - 1];
    function waitForSummary() {
        const summary = document.querySelector('[data-summary-first-attempt]');
        if (summary && summary.textContent === summary.dataset.summaryFirstAttempt) {
            done(true);
            return;
        }
        requestAnimationFrame(waitForSummary);
    }
    waitForSummary();
"#;

pub const READ_SPEECH_EVENTS: &str = r#"
    try {
        const events = JSON.parse(sessionStorage.getItem('zipeek.e2e.speech.events.v1') || '[]');
        return Array.isArray(events) ? events : [];
    } catch (_) {
        return [];
    }
"#;

pub const READ_VIEWPORT_SIZE: &str = "return [window.innerWidth, window.innerHeight];";
