import { Bucket } from './bucket';

export interface UserAccessKey {
    id: string;
    name: string;
    userId: string;
    publicKeyPem: string;
    fingerprint: string;
    apiAccess: boolean;
    createdAt: string;
    buckets: Bucket[];
};

