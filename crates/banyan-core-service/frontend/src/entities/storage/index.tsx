export class StorageLimits {
    constructor(
        public softLimit: number = 0,
        public hardLimit: number = 0,
        public size: number = 0,
    ) { };
};

export class StorageUsage {
    constructor(
        public hotStorage: number = 0,
        public archivalStorage: number = 0,
    ) { };
};