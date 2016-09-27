const chai = require('chai');
chai.use(require('chai-as-promised'));
chai.should();

const {join} = require('path');
const Watchdog = require('../../main/watchdog').default;
const config = require('../../main/config').DEFAULT_CONFIG;

describe('Watchdog.create()', () => {
    it('creates a new Watchdog instance', () => {
        return Watchdog.create(0, join(__dirname, 'watchdog_test.js'), config).should.be.ok;
    });
});
