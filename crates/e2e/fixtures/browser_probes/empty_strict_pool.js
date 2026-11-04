const post = Worker.prototype.postMessage;
window.strictPoolQueries = 0;
Worker.prototype.postMessage = function (message, ...rest) {
    if (message.type === 'CREATE_PREPARED' && message.data[1].startsWith('SELECT * FROM words WHERE hsk3_level BETWEEN')) {
        if (window.strictPoolQueries++ % 3 === 0) {
            message = {...message, data: [message.data[0], message.data[1].replace(' ORDER BY id ASC', ' AND FALSE ORDER BY id ASC')]};
        }
    }
    return post.call(this, message, ...rest);
};
