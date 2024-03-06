import { createContext, useContext, useEffect, useState } from 'react';

import ECCKeystore from '@utils/crypto/ecc/keystore';
import { EscrowedKeyMaterial, PrivateKeyMaterial } from '@utils/crypto/types';
import { AuthClient } from '@/api/auth';
import { useAppDispatch, useAppSelector } from '../store';
import { destroyLocalKey, getLocalKey } from '../utils';
import { unwrapResult } from '@reduxjs/toolkit';
import { getEscrowedKeyMaterial, getUser } from '../store/session/actions';
import { useModal } from './modals';
import { setEscrowedKeyMaterial } from '../store/session/slice';

// The name of the keystore
const KEY_STORE_NAME_PREFIX = 'banyan-key-cache';

export const KeystoreContext = createContext<{
	// Whether the user's keystore has been initialized
	keystoreInitialized: boolean;
	// Initialize a keystore based on the user's passphrase
	initializeKeystore: (passkey: string) => Promise<void>;
	// Get the user's Encryption Key Pair
	getEncryptionKey: () => Promise<{ privatePem: string, publicPem: string }>;
	// Get the user's API Key Pair
	getApiKey: () => Promise<{ privatePem: string, publicPem: string }>;
	// Purge the keystore from storage
	purgeKeystore: () => Promise<void>;
	isLoading: boolean,
	isLoggingOut: boolean;
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
	isLoggingOut: false
});

export const KeystoreProvider = ({ children }: any) => {
	const dispatch = useAppDispatch();
	const { escrowedKeyMaterial } = useAppSelector(state => state.session);
	const { openEscrowModal } = useModal();

	// External State
	const [keystoreInitialized, setKeystoreInitialized] = useState<boolean>(false);
	const [isLoading, setIsLoading] = useState<boolean>(true);
	const [isLoggingOut, setIsLoggingOut] = useState<boolean>(false);

	// Internal State
	const api = new AuthClient();
	const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
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
		(async () => {
			try {
				const ks = await ECCKeystore.init({
					storeName: KEY_STORE_NAME_PREFIX,
				});
				ks.clear();
				setKeystore(ks);
				let localKey = getLocalKey();
				try {
					await ks.retrieveCachedPrivateKeyMaterial(
						localKey.key, localKey.id
					);
					setKeystoreInitialized(true);
					console.log("createKeystore: using cached key");
				} catch (err) {
					console.log("No valid cached key material found for this session");
				}
				setIsLoading(false);
			} catch (error: any) {
				setError("Error creating keystore: " + error.message);
				throw new Error(error.message);
			}
		})();
	}, []);

	// Initialize a keystore based on the user's passphrase
	const initializeKeystore = async (passkey: string): Promise<void> => {
		let privateKeyMaterial: PrivateKeyMaterial;
		// TODO: better error handling
		if (!keystore) {
			setError('No keystore');
			throw new Error('Keystore not initialized');
		};

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
		return {
			privatePem: keyMaterial.encryptionPrivateKeyPem,
			publicPem: escrowedKeyMaterial.encryptionPublicKeyPem
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

		return {
			privatePem: privateKeyMaterial.apiPrivateKeyPem,
			publicPem: escrowedKeyMaterial.apiPublicKeyPem
		};
	};

	// Purge the keystore from storage
	const purgeKeystore = async (): Promise<void> => {
		setIsLoading(true);
		setIsLoggingOut(true);
		if (keystore) {
			await keystore.clear();
		}
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
		// Escrow the user's private key material
		await api
			.escrowDevice(escrowedKeyMaterial)
			.then(() => {
				// Set the escrowed key material in the context state and cookies
				dispatch(setEscrowedKeyMaterial(escrowedKeyMaterial))
			})
			.catch((err) => {
				throw new Error("Error escrowing device: " + err.message);
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

	useEffect(() => {
		(async () => {
			setIsLoading(true);
			try {
				unwrapResult(await dispatch(getUser()));
			} catch (error: any) {
				await purgeKeystore();
				window.location.href = '/login';
				return;
			};

			try {
				unwrapResult(await dispatch(getEscrowedKeyMaterial()));
			} catch (error: any) {
				openEscrowModal(false);
			};
			setIsLoading(false);
		})()
	}, []);

	return (
		<KeystoreContext.Provider
			value={{
				keystoreInitialized,
				isLoggingOut,
				getEncryptionKey,
				getApiKey,
				initializeKeystore,
				purgeKeystore,
				isLoading,
			}}
		>
			{children}
		</KeystoreContext.Provider>
	);
};

export const useKeystore = () => useContext(KeystoreContext);
