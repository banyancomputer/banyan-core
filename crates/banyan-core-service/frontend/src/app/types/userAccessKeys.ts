import { Bucket } from "./bucket";

export interface UserAccessKey {
	id: string;
	name: string;
	userId: string;
	pem: string;
	fingerprint: string;
	apiAccess: boolean;
	buckets: Bucket[];
};

