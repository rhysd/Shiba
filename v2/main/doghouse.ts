import Watchdog from './watchdog';
import log from './log';

export default class Doghouse {
    public dogs: {[id: number]: Watchdog};
    private id: number;

    constructor(public config: AppConfig) {
        log.debug('Doghouse created');
        this.dogs = {};
        this.id = 0;
    }

    newDog(p: string) {
        log.debug('New watchdog will be created with id', this.id, p);
        return Watchdog.create(this.id++, p, this.config).then(w => {
            this.dogs[w.id] = w;
            log.debug('New watchdog was created with id', w.id, 'Current dogs:', this.dogs);
            return w;
        });
    }

    removeDog(dog_or_id: Watchdog | number) {
        if (typeof dog_or_id === 'number') {
            if (!this.dogs[dog_or_id]) {
                log.debug('Watchdog to remove was not found for id', dog_or_id);
                return;
            }
            this.dogs[dog_or_id].stop();
            delete this.dogs[dog_or_id];
            log.debug('Watchdog was removed. id:', dog_or_id);
        } else {
            const id = dog_or_id.id;
            if (!this.dogs[id]) {
                log.debug('Watchdog to remove was not found for id', id);
                return;
            }
            dog_or_id.stop();
            delete this.dogs[id];
            log.debug('Watchdog was removed. id:', id);
        }
    }
}
