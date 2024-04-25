
export interface UserKeyAccess {
	id: string;
	name: string;
	user_id: string;
	pem: string;
	fingerprint: string;
	api_access: boolean;
	bucket_ids: string[];
};

