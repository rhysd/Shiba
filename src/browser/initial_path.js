'use strict';
let fs = require('fs');
let path = require('path');

module.exports = function() {
    let last_arg = process.argv[process.argv.length-1];
    try {
        if (fs.statSync(last_arg).isDirectory()) {
            return path.resolve(last_arg);
        } else {
            return process.cwd();
        }
    } catch (e) {
        console.log(last_arg + ' is not a directory.');
        return process.cwd();
    }
};
