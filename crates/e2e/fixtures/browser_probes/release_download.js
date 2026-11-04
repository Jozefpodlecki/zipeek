const done = arguments[arguments.length - 1];
        function release() {
            if (window.releaseDictionary) { window.releaseDictionary(); done(true); }
            else { requestAnimationFrame(release); }
        }
        release();
