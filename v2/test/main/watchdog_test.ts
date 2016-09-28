import test from 'ava';
import {join} from 'path';
import {spy} from 'sinon';
import {sync as touch} from 'touch';
import {sync as mkdir} from 'mkdirp';
import {sync as rimraf} from 'rimraf';
import Watchdog from '../../main/watchdog';
import {DEFAULT_CONFIG as config} from '../../main/config';

function wait(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const TEST_DIR = join(__dirname, '_test');
const TEST_FILE1 = join(TEST_DIR, 'test1.md');
const TEST_FILE2 = join(TEST_DIR, 'test2.md');

test.before(t => {
    mkdir(TEST_DIR);
    touch(TEST_FILE1);
    touch(TEST_FILE2);
});

test.after(t => {
    rimraf(TEST_DIR);
});

test('Watchdog.create create a new watcher', async(t) => {
    const dog = await Watchdog.create(0, TEST_FILE1, config);
    t.false(dog.started());
    t.is(dog.id, 0);
    t.is(dog.target.path, TEST_FILE1);
});

test('Watchdog watches file changes', async(t) => {
    const dog = await Watchdog.create(0, TEST_FILE1, config);
    const rcb = spy();
    dog.on('ready', rcb);
    await dog.start();
    t.true(rcb.called);
    t.true(dog.started());

    let ucb = spy();
    dog.on('update', ucb);
    touch(TEST_FILE1);
    await wait(500);
    t.true(ucb.called);
    t.is(ucb.args[0][0], TEST_FILE1);

    ucb = spy();
    dog.on('update', ucb);
    touch(TEST_FILE2);
    await wait(500);
    t.false(ucb.called);

    dog.stop();
    t.false(dog.started());
});

test('Watchdog watches all files under a directory', async(t) => {
    const dog = await Watchdog.create(0, TEST_DIR, config);
    await dog.start();

    let ucb = spy();
    dog.on('update', ucb);
    touch(TEST_FILE1);
    await wait(500);
    t.true(ucb.called);
    t.is(ucb.args[0][0], TEST_FILE1);

    ucb = spy();
    dog.on('update', ucb);
    touch(TEST_FILE2);
    await wait(500);
    t.true(ucb.called);
    t.is(ucb.args[0][0], TEST_FILE2);

    dog.stop();
});

test('Watchdog.stop() stops watching', async(t) => {
    const dog = await Watchdog.create(0, TEST_FILE1, config);
    await dog.start();

    // XXX: Stopping just after starting sometimes makes Watchdog frozen.
    await wait(100);

    dog.stop();

    const ucb = spy();
    dog.on('update', ucb);
    touch(TEST_FILE1);
    await wait(500);
    t.false(ucb.called);
});

test('Watchdog watches newly created files under a directory', async(t) => {
    const dog = await Watchdog.create(0, TEST_DIR, config);
    await dog.start();

    const ucb = spy();
    dog.on('update', ucb);
    const f = join(TEST_DIR, 'test3.md');
    touch(f);
    await wait(500);
    t.true(ucb.called);
    t.is(ucb.args[0][0], f);
    dog.stop();
});

test('Multiple instances are supported', async(t) => {
    const dog1 = await Watchdog.create(0, TEST_FILE1, config);
    await dog1.start();

    const dog2 = await Watchdog.create(0, TEST_FILE2, config);
    await dog2.start();

    const ucb1 = spy();
    dog1.on('update', ucb1);

    const ucb2 = spy();
    dog2.on('update', ucb2);

    touch(TEST_FILE1);
    await wait(500);

    t.true(ucb1.called);
    t.false(ucb2.called);

    ucb1.called = false;
    touch(TEST_FILE2);
    await wait(500);
    t.false(ucb1.called);
    t.true(ucb2.called);

    dog1.stop();
    dog2.stop();
});

test.only('Invalid path occurs an error', async(t) => {
    t.throws(Watchdog.create(0, '/path/to/unknown/file', config), Error);
});
