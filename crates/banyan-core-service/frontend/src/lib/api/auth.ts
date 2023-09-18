import { b64UrlEncode } from '../../utils/b64';
import { DeviceApiKey, EscrowedDevice } from '@/lib/interfaces';
import { EscrowedKeyMaterial } from '../crypto/types';
import { publicPemUnwrap } from '@/utils';

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
};

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
    public escrowDevice = async(
        request: EscrowedKeyMaterial 
    ): Promise<EscrowedDevice> => {
        const url = `${this.url}/auth/device/escrow`;
        const body = JSON.stringify(request);
        const opts = {
            method: 'POST',
            body,
        };

        return await fetchJson<EscrowedDevice>(url, opts);
    };

    /**
	 * Get the escrowed key material for a user
	 */
    public readEscrowedDevice = async(): Promise<EscrowedDevice | null> => {
        const url = `${this.url}/auth/device/escrow`;
        const opts = {
            method: 'GET',
        };
        const result = await fetchJson<EscrowedDevice>(url, opts).catch((e) => {
            console.log('Error reading escrowed key material: ', e);

            return null;
        });

        return result;
    };

    /* Api Key Lifecycle */

    /**
	 * Register a device api key for a user
	 * @param spki - the public key of the device's API key in PEM format
	 * @return void
	 */
    public registerDeviceApiKey = async(pem: string): Promise<DeviceApiKey> => {
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
    public readDeviceApiKeys = async(): Promise<DeviceApiKey[]> => {
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
