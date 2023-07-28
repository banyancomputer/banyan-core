import * as requests from './requests';
import * as responses from './responses';

/**
 * API Client for our web client against our NextAuth server
 */
export class ClientApi {
	url: string;
	constructor(url?: string) {
		this.url = url ?? '/api';
	}

	/* Escrowed Key Lifecycle */

	/**
	 * Initialize (or update) an Account with an Escrowed Device Key Pair and Associated public keys
	 * @param escrowed_device - the escrowed device key material to be associated with the user's account
	 */
	public escrowDevice = async (
		request: requests.EscrowDevice
	): Promise<void> => {
		const url = `${this.url}/auth/device/escrow`;
		const body = JSON.stringify(request);
		const opts = {
			method: 'POST',
			body,
		};
		const result = await fetchStatus(url, opts);
		if (result !== 200) {
			throw new Error('Error escrowing key pair');
		}
	};

	/**
	 * Get the escrowed key material for a user
	 */
	public readEscrowedDevice =
		async (): Promise<responses.GetEscrowedDevice | null> => {
			const url = `${this.url}/auth/device/escrow`;
			const opts = {
				method: 'GET',
			};
			const result = await fetchJson<responses.GetEscrowedDevice>(
				url,
				opts
			).catch((e) => {
				console.log('Error reading escrowed key material: ', e);
				return null;
			});
			return result;
		};

	/* Api Key Lifecycle */

	/**
	 * Register a device api key for a user
	 * @param fingerprint - the fingerprint of the device
	 * @param pem - the public key of the device
	 * @return void
	 */
	public registerDeviceApiKey = async (
		fingerprint: string,
		pem: string
	): Promise<void> => {
		const url = `${this.url}/auth/device/resgister?fingerpint=${fingerprint}&pem=${pem}`;
		const opts = {
			method: 'GET',
		};
		await fetchStatus(url, opts).catch((e) => {
			console.log('Error registering device api key: ', e);
			throw new Error('Error registering device api key');
		});
	};

	/**
	 * Get the device api keys for a user
	 * @return the device api keys for a user
	 */
	public readDeviceApiKeys = async (): Promise<responses.GetDeviceApiKeys> => {
		const url = `${this.url}/auth/device`;
		const opts = {
			method: 'GET',
		};
		return await fetchJson<responses.GetDeviceApiKeys>(url, opts);
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
