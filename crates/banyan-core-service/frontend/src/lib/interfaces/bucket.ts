export interface BucketFile {
    [key: string]: any;
    name: string;
    metadata: {
        [key: string]:string | number;
        created: string;
        modified: string;
        size: number;
    };
};

export interface BucketKey {
    id: string;
    bucket_id: string;
    pem: string;
    approved: boolean;
};

export interface Bucket {
    id: string;
    name: string;
    type: string;
    storage_class: string;
    files: BucketFile[];
    keys: BucketKey[];
};

export interface Metadata {
    id: string;
    bucket_id: string;
    path: string;
    type: string;
    cid: string;
    size: string;
    versions: any[];
    created_at: string;
    updated_at: string;
};

export class MockBucket {
    public id = '';
    public name = '';
    public type = '';
    public storage_class = '';
    public files = [];
    public keys = [];
};

export interface BucketSnapshot {
    id: string;
    bucket_id: string;
    snapshot_type: string;
    version: string;
};
