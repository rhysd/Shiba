'use strict';
let fs = require('fs');
let path = require('path');

module.exports = function() {
    let last_arg = process.argv[process.argv.length-1];
    if (fs.existsSync(last_arg)) {
        return path.resolve(last_arg);
    } else {
        return process.cwd();
    }
};
