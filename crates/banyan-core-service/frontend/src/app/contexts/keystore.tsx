import { createContext, useContext, useEffect, useState } from 'react';
import ECCKeystore from '@/app/lib/crypto/ecc/keystore';
import {
	fingerprintDeviceApiPublicKeyPem,
	hexFingerprint,
} from '@/app/lib/crypto/utils';
import { DeviceApiKey } from '@/app/types';
import { ClientApi } from '@/app/lib/api/auth';
import { EscrowedKeyMaterial, PrivateKeyMaterial } from '@/app/lib/crypto/types';
import { useSession } from '@/app/contexts/session';

// The name of the keystore
const KEY_STORE_NAME_PREFIX = 'banyan-key-cache';

export const KeystoreContext = createContext<{
	// External State

	// Whether the user's keystore has been initialized
	keystoreInitialized: boolean;

	// External Methods

	// Initialize a keystore based on the user's passphrase
	initializeKeystore: (passkey: string) => Promise<void>;
	// Get the user's Encryption Key Pair
	getEncryptionKey: () => Promise<{ privatePem: string, publicPem: string }>;
	// Get the user's API Key Pair
	getApiKey: () => Promise<{ privatePem: string, publicPem: string }>;
	// Purge the keystore from storage
	purgeKeystore: () => Promise<void>;
	isLoading: boolean,
	escrowedKeyMaterial: EscrowedKeyMaterial | null;
}>({
	keystoreInitialized: false,
	getEncryptionKey: async () => {
		throw new Error('Keystore not initialized');
	},
	getApiKey: async () => {
		throw new Error('Keystore not initialized');
	},
	initializeKeystore: async (_passkey: string) => { },
	purgeKeystore: async () => { },
	isLoading: false,
	escrowedKeyMaterial: null
});

export const KeystoreProvider = ({ children }: any) => {
	const { userData, getLocalKey, destroyLocalKey } = useSession();

	// External State
	const [keystoreInitialized, setKeystoreInitialized] = useState<boolean>(false);
	const [isLoading, setIsLoading] = useState<boolean>(true);

	// Internal State
	const api = new ClientApi();
	const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
	const [escrowedKeyMaterial, setEscrowedKeyMaterial] = useState<EscrowedKeyMaterial | null>(null);
	const [error, setError] = useState<string | null>(null);

	/* Effects */

	// Handle errors
	useEffect(() => {
		if (error) {
			console.error(error);
		}
	}, [error]);

	// Handle creating the keystore if it doesn't exist
	// Occurs on context initialization
	useEffect(() => {
		const createKeystore = async () => {
			console.log("createKeystore");
			try {
				const ks = await ECCKeystore.init({
					storeName: KEY_STORE_NAME_PREFIX,
				});
				ks.clear();
				setKeystore(ks);
				// Try and initialize the keystore with cached key material
				let initialized = false;
				let localKey = getLocalKey();
				try {
					await ks.retrieveCachedPrivateKeyMaterial(
						localKey.key, localKey.id
					);
					initialized = true;
				} catch (err) {
					console.log("No valid cached key material found for this session");
				} finally {
					setKeystoreInitialized(initialized);
					setIsLoading(false);
				}
			} catch (error: any) {
				setError("Error creating keystore: " + error.message);
				throw new Error(error.message);
			}
		};
		if (!keystore) {
			createKeystore()
		}
	}, [keystore]);

	// Handle loading the escrowed key material from the Next Auth session
	// Occurs on update to the session context
	useEffect(() => {
		console.log("Initializing Keystore with UserData: ", userData);
		if (userData) {
			setEscrowedKeyMaterial(userData.escrowedKeyMaterial);
			!userData.escrowedKeyMaterial && purgeKeystore();
		}
	}, [userData]);

	// Initialize a keystore based on the user's passphrase
	const initializeKeystore = async (passkey: string): Promise<void> => {
		let privateKeyMaterial: PrivateKeyMaterial;
		// TODO: better error handling
		if (!keystore) {
			setError('No keystore');
			throw new Error('Keystore not initialized');
		}
		try {
			if (escrowedKeyMaterial) {
				privateKeyMaterial = await recoverDevice(passkey);
			} else {
				privateKeyMaterial = await escrowDevice(passkey);
			}
			let localKey = getLocalKey();
			// Cache the key material encrypted with the session key
			await keystore.cachePrivateKeyMaterial(
				privateKeyMaterial,
				localKey.key,
				localKey.id
			);
			setKeystoreInitialized(true);
		} catch (err: any) {
			console.error(err);
			setError("Error initializing keystore: " + err.message);
			throw new Error(err.message);
		}
	};

	// Get the user's Encryption Key Pair as a Public / Private PEM combo
	const getEncryptionKey = async (): Promise<{ privatePem: string, publicPem: string }> => {
		if (!keystore) {
			setError('No keystore');
			throw new Error('No keystore');
		}
		if (!keystoreInitialized) {
			setError('Keystore not initialized');
			throw new Error('Keystore not initialized');
		}
		if (!escrowedKeyMaterial) {
			setError('Missing escrowed data');
			throw new Error('Missing escrowed data');
		}
		let localKey = getLocalKey();
		const keyMaterial = await keystore.retrieveCachedPrivateKeyMaterial(
			localKey.key, localKey.id
		);
		// Get pems to return
		let publicPem = escrowedKeyMaterial.encryptionPublicKeyPem;
		let privatePem = keyMaterial.encryptionPrivateKeyPem;
		return {
			privatePem,
			publicPem
		};
	};

	// Get the user's API Key as a Private / Public PEM combo
	const getApiKey = async (): Promise<{ privatePem: string, publicPem: string }> => {
		if (!keystore || !keystoreInitialized) {
			setError('Keystore not initialized');
			throw new Error('Keystore not initialized');
		}
		if (!escrowedKeyMaterial) {
			setError('Missing escrowed data');
			throw new Error('Missing escrowed data');
		}
		let localKey = getLocalKey();
		const privateKeyMaterial = await keystore.retrieveCachedPrivateKeyMaterial(
			localKey.key, localKey.id
		);
		// Get pems to return
		let publicPem = escrowedKeyMaterial.apiPublicKeyPem;
		let privatePem = privateKeyMaterial.apiPrivateKeyPem;
		return {
			privatePem,
			publicPem
		};
	};

	// Purge the keystore from storage
	const purgeKeystore = async (): Promise<void> => {
		setIsLoading(true);
		if (!keystore) {
			setError('No keystore');
			throw new Error('No keystore');
		}
		await keystore.clear();
		// Purge the local key cookie
		destroyLocalKey();
		setKeystoreInitialized(false);
		setTimeout(() => {
			setIsLoading(false);
		}, 500);
	};

	/* Helpers */

	// Register a new user in firestore
	const escrowDevice = async (passphrase: string): Promise<PrivateKeyMaterial> => {
		if (!keystore) {
			setError('No keystore');
			throw new Error('No keystore');
		}
		const keyMaterial = await keystore.genKeyMaterial();
		const privateKeyMaterial = await keystore.exportPrivateKeyMaterial(keyMaterial);
		const escrowedKeyMaterial = await keystore.escrowKeyMaterial(
			keyMaterial,
			passphrase
		);
		setEscrowedKeyMaterial(escrowedKeyMaterial);
		// Escrow the user's private key material
		await api
			.escrowDevice(escrowedKeyMaterial)
			.then((resp) => {
				setEscrowedKeyMaterial(resp);
			})
			.catch((err) => {
				throw new Error("Error escrowing device: " + err.message);
			});

		const apiKeyFingerprint = await fingerprintDeviceApiPublicKeyPem(escrowedKeyMaterial.apiPublicKeyPem)
			.then(hexFingerprint).catch((err) => {
				throw new Error('Error fingerprinting API key: ' + err.message);
			});

		// Register the user's public key material
		await api
			.registerDeviceApiKey(escrowedKeyMaterial.apiPublicKeyPem)
			.then((resp: DeviceApiKey) => {
				if (resp.fingerprint !== apiKeyFingerprint) {
					setError('Fingerprint mismatch on registration');
					throw new Error('Fingerprint mismatch on registration');
				}
			})
			.catch((err) => {
				setError(err.message);
				throw new Error(err.message);
			});
		return privateKeyMaterial;
	};

	// Recovers a device's private key material from escrow
	const recoverDevice = async (passphrase: string) => {
		if (!keystore) {
			setError('No keystore');
			throw new Error('No keystore');
		}
		if (!escrowedKeyMaterial) {
			setError('No escrowed device');
			throw new Error('No escrowed device');
		}
		return await keystore.recoverKeyMaterial(
			escrowedKeyMaterial,
			passphrase
		);
	};

	return (
		<KeystoreContext.Provider
			value={{
				keystoreInitialized,
				getEncryptionKey,
				getApiKey,
				initializeKeystore,
				purgeKeystore,
				isLoading,
				escrowedKeyMaterial
			}}
		>
			{children}
		</KeystoreContext.Provider>
	);
};

export const useKeystore = () => useContext(KeystoreContext);
