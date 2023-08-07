/* DB lifecycle errors */

export class BadModelFormat extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'BadModelFormat';
    }
}
