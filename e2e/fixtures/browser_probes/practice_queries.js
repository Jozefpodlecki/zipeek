const post = Worker.prototype.postMessage;
window.practiceQueryMode = '';
Worker.prototype.postMessage = function (message, ...rest) {
    if (message.type === 'CREATE_PREPARED' && window.practiceQueryMode) {
        const mode = window.practiceQueryMode;
        window.practiceQueryMode = '';
        if (mode.includes('fail')) {
            message = {...message, data: [message.data[0], 'SELECT * FROM missing_e2e_practice_table']};
        }
        if (mode.includes('hold')) {
            window.releasePracticeQuery = () => post.call(this, message, ...rest);
            return;
        }
    }
    return post.call(this, message, ...rest);
};
