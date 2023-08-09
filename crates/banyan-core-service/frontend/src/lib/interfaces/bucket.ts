export interface Bucket {
    id: string;
    name: string;
    type: 'Backup' | 'Interactive';
    metadata: string;
};


/** TODO: delete after connection to backend */
export class MockBucket {
    constructor(
        public id: string,
        public name: string,
        public type: string,
        metadata: string = ''
    ) { }
}
