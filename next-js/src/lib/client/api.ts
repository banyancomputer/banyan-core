import { IPublicKey, IEscrowedKey } from '@/lib/db/entities';

/**
 * API Client for our web client. Relies on the current user's session for authentication.
 * @property create - create a object of the given type for the user
 * @property read - read an object of the given type for the user. Optionally, pass an id to read a specific object,
 * if the id is not the user's id.
 */
class Api<T> {
	url: string;
	constructor(url: string) {
		this.url = url;
		this.create = async (data: Partial<T> = {}) => {
			const url = `${this.url}`;
			const body = JSON.stringify(data);
			const opts = {
				method: 'POST',
				body,
			};
			return await fetchJson<T>(url, opts);
		};
	}

	public create = async (data: Partial<T> = {}) => {
		const url = `${this.url}`;
		const body = JSON.stringify(data);
		const opts = {
			method: 'POST',
			body,
		};
		return await fetchJson<T>(url, opts);
	};

	read = async (id?: string): Promise<T> => {
		const url = `${this.url}?id=${id}`;
		return await fetchJson<T>(url);
	};
}

/* Web Client API access via pages/api/keys/public.ts */
export const PublicKeyApi = new Api<IPublicKey>('/api/keys/public');
/* Web Client API access via pages/api/keys/escrowed.ts */
export const EscrowedKeyApi = new Api<IEscrowedKey>('/api/keys/escrowed');

async function fetchJson<T>(url: string, opts?: {}): Promise<T> {
	try {
		const response = await fetch(url, {
			headers: {
				'Content-Type': 'application/json',
			},
			...opts,
		});
		const data = await response.json();
		return data as T;
	} catch (error) {
		throw error;
	}
}
