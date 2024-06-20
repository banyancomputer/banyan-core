import { WasmMount } from 'tomb-wasm-experimental';

export interface FileMetadata {
    [key: string]:string | number | undefined;
    created: string;
    mimeType?: string;
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

export interface BucketKey {
    id: string;
    bucket_id: string;
    pem: string;
    approved: boolean;
    fingerPrint: string;
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
        public keys: BucketKey[] = [],
    ) {}
};
