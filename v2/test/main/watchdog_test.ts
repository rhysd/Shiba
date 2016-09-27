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
    t.is(dog.id, 0);
    t.is(dog.target.path, TEST_FILE1);
});

test('Watchdog watches file change', async(t) => {
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
