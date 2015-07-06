#! /usr/bin/env node

'use strict';

var child_process = require('child_process');
var electron = require('electron-prebuilt');
var path = require('path');

var mainu = path.join(__dirname, '..', 'src', 'browser', 'mainu.js');
var argv = process.argv;
argv.unshift(mainu);
console.log(argv);

child_process.spawn(electron, argv);
