export interface BucketFile {
    [key: string]: any;
    name: string;
    metadata: {
        [key: string]:string | number;
        created: string;
        modified: string;
        size: number;
    };
}
export interface Bucket {
    id: string;
    name: string;
    bucket_type: string;
    files: BucketFile[];
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
    public bucket_type = '';
    public files = [];
};

export interface BucketSnapshot {
    id: string,
    bucket_id: string,
    snapshot_type: string,
    version: string
};

export interface BucketKey {
    id: string;
    bucket_id: string;
    pem: string;
    approved: boolean;
}