import * as requests from './requests';
import * as responses from './responses';

/**
 * API Client for our web client against our NextAuth server
 */
export class ClientApi {
	url: string;
	constructor(url?: string) {
		this.url = url ?? '/api'
	}

	/* Escrowed Key Lifecycle */

	public escrowKeyPair = async (
		request: requests.EscrowDeviceKeyPair
	): Promise<responses.EscrowDeviceKeyPair> => {
		const url = `${this.url}/auth/keys/escrow`;
		const body = JSON.stringify(request);
		const opts = {
			method: 'POST',
			body,
		};
		return await fetchJson<responses.EscrowDeviceKeyPair>(url, opts);
	};

	public readEscrowedKeyPair =
		async (): Promise<responses.EscrowDeviceKeyPair> => {
			const url = `${this.url}/auth/keys/escrow`;
			const opts = {
				method: 'GET',
			};
			return await fetchJson<responses.EscrowDeviceKeyPair>(url, opts);
		};

	/* Public Key Lifecycle */

	public registerDevicePublicKey = async (
		request: requests.RegisterDevicePublicKey
	): Promise<responses.RegisterDevicePublicKey> => {
		const url = `${this.url}/auth/keys/public`;
		const body = JSON.stringify(request);
		const opts = {
			method: 'PUT',
			body,
		};
		return await fetchJson<responses.RegisterDevicePublicKey>(url, opts);
	};

	public deleteDevicePublicKey = async (
		request: requests.DeleteDevicePublicKey
	): Promise<number> => {
		const url = `${this.url}/auth/keys/public`;
		const body = JSON.stringify(request);
		const opts = {
			method: 'DELETE',
			body,
		};
		return await fetchStatus(url, opts);
	};
}

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

async function fetchStatus(url: string, opts?: {}): Promise<number> {
	try {
		const response = await fetch(url, {
			...opts,
		});
		return response.status;
	} catch (error) {
		throw error;
	}
}
