import { WasmMount } from 'tomb-wasm-experimental';

export interface FileMetadata {
    [key: string]: string | number;
    created: string;
    modified: string;
    size: number;
};

export interface BrowserObject {
    [key: string]: string | BrowserObject[] | FileMetadata;
    name: string;
    type: 'file' | 'dir';
    files: BrowserObject[];
    metadata: FileMetadata;
};

export interface BucketSnapshot {
    id: string;
    bucket_id: string;
    metadata_id: string;
    state: string;
    version: string;
    size: number;
    created_at: number;
};

export class Bucket {
    constructor(
        public id: string,
        public name: string,
        public mount: WasmMount | null,
        public bucketType: string,
        public storageClass: string,
        public isSnapshotValid: boolean = false,
        public locked: boolean = false,
        public snapshots: BucketSnapshot[] = [],
        public files: BrowserObject[] = [],
    ) { }
};

export class MockBucket {
    public id = '';
    public name = '';
    public bucketType = '';
    public storageClass = '';
    public mount = {} as WasmMount;
    public files = [];
    public isSnapshotValid = false;
    public snapshots = [];
    public access = [];
    public locked = false;
};
