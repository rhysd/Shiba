import log = require('loglevel');

if (process.env.NODE_ENV === 'development') {
    log.setLevel('debug');
} else {
    log.setLevel('info');
}

export default log;
