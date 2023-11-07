import { b64UrlEncode } from '../../utils/b64';
import { DeviceApiKey, EscrowedDevice } from '@app/lib/interfaces';
import { EscrowedKeyMaterial } from '../crypto/types';
import { publicPemUnwrap } from '@app/utils';

export async function fetchJson<T>(url: string, opts?: {}): Promise<T> {
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
};

/**
 * API Client for our web client against our NextAuth server
 */
export class ClientApi {
	url: string;
	constructor(url?: string) {
		this.url = url ?? '';
	}

	/* Escrowed Key Lifecycle */

	/**
 * Initialize (or update) an Account with an Escrowed Device Key Pair and Associated public keys
 * @param escrowed_device - the escrowed device key material to be associated with the user's account
 */
	public escrowDevice = async (
		request: EscrowedKeyMaterial
	): Promise<void> => {
		const url = `${this.url}/api/v1/auth/create_escrowed_device`;
		const body = JSON.stringify({
			api_public_key_pem: request.apiPublicKeyPem,
			encryption_public_key_pem: request.encryptionPublicKeyPem,
			encrypted_private_key_material: request.encryptedPrivateKeyMaterial,
			pass_key_salt: request.passKeySalt
		});
		const opts = {
			headers: {
				'Content-Type': 'application/json',
			},
			method: 'POST',
			body,
		};

		let status = await fetchStatus(url, opts);
		if (status != 200) {
			throw new Error("failed to escrow device")
		}
	};

	/* Api Key Lifecycle */

	/**
 * Register a device api key for a user
 * @param spki - the public key of the device's API key in PEM format
 * @return void
 */
	public registerDeviceApiKey = async (pem: string): Promise<DeviceApiKey> => {
		const spki = publicPemUnwrap(pem);
		const urlSpki = b64UrlEncode(spki);
		const url = `${this.url}/auth/device/register?spki=${urlSpki}`;
		const opts = {
			method: 'GET',
		};

		return await fetchJson<DeviceApiKey>(url, opts);
	};

	/**
 * Get the device api keys for a user
 * @return the device api keys for a user
 */
	public readDeviceApiKeys = async (): Promise<DeviceApiKey[]> => {
		const url = `${this.url}/auth/device`;
		const opts = {
			method: 'GET',
		};

		return await fetchJson<DeviceApiKey[]>(url, opts);
	};
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
