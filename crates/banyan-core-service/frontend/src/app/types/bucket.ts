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

export interface BucketAccess {
    user_key_id: string;
    bucket_id: string;
    state: string;
    fingerprint: string;
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

export interface Bucket {
    id: string;
    name: string;
    mount: WasmMount | null;
    bucketType: string;
    storageClass: string;
    files: BrowserObject[];
    snapshots: BucketSnapshot[];
    isSnapshotValid: boolean;
    access: BucketAccess[];
    locked: boolean;
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
