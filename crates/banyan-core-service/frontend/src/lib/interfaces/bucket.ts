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


export class MockBucket {
    public id = '';
    public name = '';
    public bucket_type = '';
    public files = [];
}

export interface BucketSnapshot {
    id: string,
    bucket_id: string,
    snapshot_type: string,
    version: string
}