import Watchdog from './watchdog';

export default class Doghouse {
    public dogs: {[id: number]: Watchdog};
    private id: number;

    constructor(public config: AppConfig) {
        this.dogs = {};
        this.id = 0;
    }

    newDog(p: string) {
        return Watchdog.create(this.id++, p, this.config).then(w => {
            this.dogs[w.id] = w;
            return w;
        });
    }

    removeDog(dog_or_id: Watchdog | number) {
        if (typeof dog_or_id === 'number') {
            if (!this.dogs[dog_or_id]) {
                return;
            }
            this.dogs[dog_or_id].stop();
            delete this.dogs[dog_or_id];
        } else {
            const id = dog_or_id.id;
            if (!this.dogs[id]) {
                return;
            }
            dog_or_id.stop();
            delete this.dogs[id];
        }
    }
}
