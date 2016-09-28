import test from 'ava';
import {join} from 'path';
import {sync as touch} from 'touch';
import {sync as rimraf} from 'rimraf';
import Doghouse from '../../main/doghouse';
import {DEFAULT_CONFIG as config} from '../../main/config';

const TEST_FILE = join(__dirname, 'test.md');

function wait(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

test.before(t => {
    touch(TEST_FILE);
});

test.after(t => {
    rimraf(TEST_FILE);
});

test('Doghouse can create a new watcher', async(t) => {
    const house = new Doghouse(config);
    t.is(Object.keys(house.dogs).length, 0);

    const dog = await house.newDog(TEST_FILE);
    t.is(dog.id, 0);
    t.false(dog.started());
    t.is(Object.keys(house.dogs).length, 1);

    const dog2 = await house.newDog(TEST_FILE);
    t.is(dog2.id, 1);
    t.false(dog2.started());
    t.is(Object.keys(house.dogs).length, 2);
});

test('Doghouse can remove dogs', async(t) => {
    const house = new Doghouse(config);
    const dog = await house.newDog(TEST_FILE);
    const dog2 = await house.newDog(TEST_FILE);
    t.is(Object.keys(house.dogs).length, 2);

    await dog.start();
    await wait(100);
    t.true(dog.started());
    house.removeDog(dog);

    t.false(dog.started());
    t.is(Object.keys(house.dogs).length, 1);

    house.removeDog(dog2.id);
    t.is(Object.keys(house.dogs).length, 0);
});
