import { createContext, useState, useEffect, useContext } from 'react';
import { ClientApi } from '@/lib/api/auth';
import { EscrowedDevice } from '@/lib/interfaces';
import ECCKeystore from 'banyan-webcrypto-experiment/ecc/keystore';
import { clear as clearIdb } from 'banyan-webcrypto-experiment/idb';
import { useSession } from 'next-auth/react';
import { Session } from 'next-auth';
import { publicPemWrap, publicPemUnwrap } from '@/lib/utils';

const KEY_STORE_NAME_PREFIX = 'key-store';
const EXCHANGE_KEY_PAIR_NAME = 'exchange-key-pair';
const WRITE_KEY_PAIR_NAME = 'write-key-pair';
const ESCROW_KEY_NAME = 'escrow-key';

export const KeystoreContext = createContext<{
	// External State

	// Whether the user's keystore has been initialized
	keystoreInitialized: boolean;

	// External Methods

	// Initialize a keystore based on the user's passphrase
	initializeKeystore: (passkey: string) => Promise<void>;
	// Purge the keystore from storage
	purgeKeystore: () => Promise<void>;
	// Get the public key's fingerprint
	getFingerprint: () => Promise<string>;
}>({
	keystoreInitialized: false,
	initializeKeystore: async (passkey: string) => {},
	purgeKeystore: async () => {},
	getFingerprint: async () => '',
});

export const KeystoreProvider = ({ children }: any) => {
	/* State */
	const { data: session } = useSession();

	// External State
	const [keystoreInitialized, setKeystoreInitialized] =
		useState<boolean>(false);

	// Internal State
	const api = new ClientApi();
	const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
	const [escrowedDevice, setEscrowedDevice] = useState<EscrowedDevice | null>(
		null
	);
	const [apiKeyPem, setApiKeyPem] = useState<string | null>(null);
	const [encryptionKeyPem, setEncryptionKeyPem] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	/* Effects */

	// Handle errors
	useEffect(() => {
		if (error) {
			console.error(error);
		}
	}, [error]);

	// Set the keystore and escrowedDevice state based on the session
	useEffect(() => {
		const createKeystore = async (session: Session) => {
			// Initialize a keystore pointed by the user's uid
			const storeName = KEY_STORE_NAME_PREFIX + '-' + session.providerId;
			// Defaults are fine here
			const ks = await ECCKeystore.init({ storeName });
			setKeystore(ks);
		};

		if (session) {
			createKeystore(session);
			// getEscrowedDevice(session);
		}
	}, [session]);

	// Decide whether the user's keystore has been initialized
	useEffect(() => {
		const tryInitKeystore = async (ks: ECCKeystore) => {
			if (
				(await ks.keyExists(ESCROW_KEY_NAME)) &&
				(await ks.keyPairExists(EXCHANGE_KEY_PAIR_NAME)) &&
				(await ks.keyPairExists(WRITE_KEY_PAIR_NAME))
			) {
				setKeystoreInitialized(true);
				return true;
			}
			return false;
		};
		const getEscrowedDevice = async () => {
			const resp = await api.readEscrowedDevice().catch((err) => {
				return undefined;
			});
			if (resp) {
				setEscrowedDevice(resp.escrowed_device);
				setEncryptionKeyPem(resp.encryption_key_pem);
				setApiKeyPem(resp.api_key_pem);
			}
		};
		if (keystore) {
			tryInitKeystore(keystore).then((init) => {
				if (!init) {
					getEscrowedDevice();
				}
			});
		}
	}, [keystore]);

	// // Set the isRegistered state if the user has an encrypted private key in the db
	// useEffect(() => {
	// 	const check = async (session: Session) => {
	// 		const resp = await api.readEscrowedKeyPair().catch((err) => {
	// 			return undefined;
	// 		});
	// 		if (resp) {
	// 			setIsRegistered(true);
	// 			setEscrowedDeviceKeyPair(resp.escrowed_device_key_pair);
	// 		}
	// 	};
	// 	if (session) {
	// 		check(session);
	// 	}
	// }, [session]);

	/* Methods */

	// Initialize a keystore based on the user's passphrase
	const initializeKeystore = async (passkey: string): Promise<void> => {
		if (escrowedDevice && encryptionKeyPem && apiKeyPem) {
			console.log('Initializing keystore with recovered escrowed data');
			await recoverKeystore(passkey);
		} else {
			console.log('Registering user');
			await escrowKeystore(passkey);
		}
	};

	// Get the ecdsa public key's fingerprint
	const getFingerprint = async (): Promise<string> => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}
		return await keystore.fingerprintPublicWriteKey();
	};

	// Purge the keystore from storage
	const purgeKeystore = async (): Promise<void> => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}
		await keystore.destroy();
		await clearIdb();
		setKeystore(null);
		setKeystoreInitialized(false);
		setEscrowedDevice(null);
		setEncryptionKeyPem(null);
		setApiKeyPem(null);
	};

	/* Helpers */

	// Register a new user in firestore
	const escrowKeystore = async (passphrase: string): Promise<void> => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}

		await keystore.genExchangeKeyPair();
		await keystore.genWriteKeyPair();
		// Derive a new passkey for the user -- this generates a random salt
		const passkey_salt = await keystore.deriveEscrowKey(passphrase);
		const wrapped_ecdh_key_pair =
			await keystore.exportEscrowedExchangeKeyPair();
		const wrapped_ecdsa_key_pair = await keystore.exportEscrowedWriteKeyPair();

		const escrowed_device = {
			ecdsa_fingerprint: await keystore.fingerprintPublicWriteKey(),
			ecdh_fingerprint: await keystore.fingerprintPublicExchangeKey(),
			wrapped_ecdsa_pkcs8: wrapped_ecdsa_key_pair.wrappedPrivateKeyStr,
			wrapped_ecdh_pkcs8: wrapped_ecdh_key_pair.wrappedPrivateKeyStr,
			passkey_salt: passkey_salt,
		};
		const api_key_pem = publicPemWrap(wrapped_ecdsa_key_pair.publicKeyStr);
		const encryption_key_pem = publicPemWrap(
			wrapped_ecdh_key_pair.publicKeyStr
		);

		await api
			.escrowDevice({
				escrowed_device,
				api_key_pem,
				encryption_key_pem,
			})
			.then(() => {
				setEscrowedDevice(escrowed_device);
				setEncryptionKeyPem(encryption_key_pem);
				setApiKeyPem(api_key_pem);
			})
			.catch((err) => {
				setError(err.message);
				setEscrowedDevice(null);
				setEncryptionKeyPem(null);
				setApiKeyPem(null);
				throw new Error(err.message);
			});
	};

	const recoverKeystore = async (passphrase: string) => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}
		if (!escrowedDevice || !encryptionKeyPem || !apiKeyPem) {
			throw new Error('Invalid escrowed data');
		}

		const {
			passkey_salt,
			wrapped_ecdsa_pkcs8,
			wrapped_ecdh_pkcs8,
			ecdsa_fingerprint,
		} = escrowedDevice;
		const ecdsa_spki = publicPemUnwrap(apiKeyPem);
		const ecdh_spki = publicPemUnwrap(encryptionKeyPem);

		await keystore.deriveEscrowKey(passphrase, passkey_salt);
		await keystore.importEscrowedWriteKeyPair({
			publicKeyStr: ecdsa_spki,
			wrappedPrivateKeyStr: wrapped_ecdsa_pkcs8,
		});
		await keystore.importEscrowedExchangeKeyPair({
			publicKeyStr: ecdh_spki,
			wrappedPrivateKeyStr: wrapped_ecdh_pkcs8,
		});

		// Check that the keystore is valid
		// Check that the fingerprint matches
		const fingerprint = await keystore.fingerprintPublicWriteKey();
		if (fingerprint !== ecdsa_fingerprint) {
			setError(
				'Keystore is invalid: ' + fingerprint + ' != ' + ecdsa_fingerprint
			);
			throw new Error(
				'Keystore is invalid: ' + fingerprint + ' != ' + ecdsa_fingerprint
			);
		}

		const msg = 'hello world';

		const ciphertext = await keystore.encrypt(msg, ecdh_spki, passkey_salt);
		const plaintext = await keystore.decrypt(
			ciphertext,
			ecdh_spki,
			passkey_salt
		);
		if (plaintext !== msg) {
			setError('Keystore is invalid: ' + plaintext + ' != ' + msg);
			throw new Error('Keystore is invalid: ' + plaintext + ' != ' + msg);
		}
		const signature = await keystore.sign(msg);
		const verified = await keystore.verify(msg, signature, ecdsa_spki);
		if (!verified) {
			setError('Keystore is invalid (signature)');
			throw new Error('Keystore is invalid (signature)');
		}
	};

	return (
		<KeystoreContext.Provider
			value={{
				keystoreInitialized,
				initializeKeystore,
				getFingerprint,
				purgeKeystore,
			}}
		>
			{children}
		</KeystoreContext.Provider>
	);
};

export const useKeystore = () => useContext(KeystoreContext);
