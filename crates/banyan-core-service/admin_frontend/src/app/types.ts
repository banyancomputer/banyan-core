export interface StorageHost {
	id: string;
	name: string;
	url: string;
	used_storage: number;
	available_storage: number;
	fingerprint: string;
	pem: string;
}

export interface StorageHostRequest {
	name: string;
	url: string;
	available_storage: number;
}

export enum DealState {
	Active = 'Active',
	Accepted = 'Accepted',
	Sealed = 'Sealed',
	Finalized = 'Finalized',
	Cancelled = 'Cancelled',
}

export interface Deal {
	id: string;
	state: DealState;
	size: number;
	accepted_by: string | null;
	accepted_at: string | null;
}

export interface User {
	id: string;
	email: string;
	verifiedEmail: boolean;
	displayName: string;
	locale: string;
	profileImage: string;
	acceptedTosAt: number | null;
}
