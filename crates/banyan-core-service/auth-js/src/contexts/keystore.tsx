import { createContext, useState, useEffect, useContext } from 'react';
import { ClientApi } from '@/lib/api/auth';
import { DeviceApiKey, EscrowedDevice } from '@/lib/interfaces';
import ECCKeystore from 'banyan-webcrypto-experiment/ecc/keystore';
import * as KeyStore from 'banyan-webcrypto-experiment/keystore/index';
import {
	fingerprintEcPublicKey,
	prettyFingerprint,
} from 'banyan-webcrypto-experiment/utils';
import { useSession } from 'next-auth/react';
import { Session } from 'next-auth';
import { publicPemWrap, publicPemUnwrap, prettyFingerprintApiKeyPem } from '@/lib/utils';

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
}>({
	keystoreInitialized: false,
	initializeKeystore: async (passkey: string) => {},
	purgeKeystore: async () => {},
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
			const ks = (await KeyStore.init({ 
				escrowKeyName: ESCROW_KEY_NAME,
				writeKeyPairName: WRITE_KEY_PAIR_NAME,
				exchangeKeyPairName: EXCHANGE_KEY_PAIR_NAME,
				storeName 
			})) as ECCKeystore;
			setKeystore(ks);
		};
		
		if (session) {
			createKeystore(session);
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
			const resp = (await api.readEscrowedDevice().catch((err) => {
				return undefined;
			})) as EscrowedDevice | undefined;
			if (resp) {
				setEscrowedDevice(resp);
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

	/* Methods */

	// Initialize a keystore based on the user's passphrase
	const initializeKeystore = async (passkey: string): Promise<void> => {
		if (escrowedDevice) {
			await recoverDevice(passkey);
		} else {
			await escrowDevice(passkey);
		}
	};

	// Purge the keystore from storage
	const purgeKeystore = async (): Promise<void> => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}
		await keystore.destroy();
		await KeyStore.clear();
		setKeystore(null);
		setKeystoreInitialized(false);
		setEscrowedDevice(null);
	};

	/* Helpers */

	// Register a new user in firestore
	const escrowDevice = async (passphrase: string): Promise<void> => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}

		await keystore.genExchangeKeyPair();
		await keystore.genWriteKeyPair();
		// Derive a new passkey for the user -- this generates a random salt
		const passKeySalt = await keystore.deriveEscrowKey(passphrase);
		const apiKeyPair = await keystore.getWriteKeyPair();

		const apiKeyFingerprint = await fingerprintEcPublicKey(
			apiKeyPair.publicKey as CryptoKey
		).then((fingerprint) => prettyFingerprint(fingerprint));
		const apiKeySpki = await keystore.exportPublicWriteKey()
		const apiKeyPem = publicPemWrap(apiKeySpki);
		const encryptionKeyPem = await keystore.exportPublicExchangeKey()
			.then((key) => key)
			.then((key) => publicPemWrap(key));

		const wrappedApiKey = await keystore.exportEscrowedPrivateWriteKey();
		const wrappedEncryptionKey = await keystore.exportEscrowedPrivateExchangeKey();

		// Escrow the user's private key material
		await api
			.escrowDevice({
				apiKeyPem,
				encryptionKeyPem,
				wrappedApiKey,
				wrappedEncryptionKey,
				passKeySalt, 	
			})
			.then((resp) => {
				setEscrowedDevice(resp);
			})
			.catch((err) => {
				setError(err.message);
				setEscrowedDevice(null);
				throw new Error(err.message);
			});

		console.log("Registering device:")
		console.log("apiKeySpki: " + apiKeySpki)

		// Register the user's public key material
		await api
			.registerDeviceApiKey(apiKeySpki)
			.then((resp: DeviceApiKey) => {
				console.log("Registered device:")
				console.log("apiKeyPem: " + resp.pem);
				console.log("apiKeyFingerprint: " + resp.fingerprint);
				if (resp.fingerprint !== apiKeyFingerprint) {
					setError('Fingerprint mismatch');
					throw new Error('Fingerprint mismatch');
				}
			})
			.catch((err) => {
				setError(err.message);
				throw new Error(err.message);
			});

		setKeystoreInitialized(true);
	};

	// Recovers a device's private key material from escrow
	const recoverDevice = async (passphrase: string) => {
		if (!keystore) {
			throw new Error('Keystore not initialized');
		}
		if (!escrowedDevice) {
			throw new Error('Invalid escrowed data');
		}
		console.log("Recovering device:")
		const {
			apiKeyPem,
			encryptionKeyPem,
			wrappedApiKey,
			wrappedEncryptionKey,
			passKeySalt
		} = escrowedDevice;

		await keystore.deriveEscrowKey(passphrase, passKeySalt);

		const apiKeySpki = publicPemUnwrap(apiKeyPem);
		const encryptionKeySpki = publicPemUnwrap(encryptionKeyPem);

		console.log("apiKeySpki: " + apiKeySpki)
		await keystore.importEscrowedWriteKeyPair(
			apiKeySpki,
			wrappedApiKey
		);
		console.log("encryptionKeySpki: " + encryptionKeySpki);
		await keystore.importEscrowedExchangeKeyPair(
			encryptionKeySpki,
			wrappedEncryptionKey
		);

		/* Validate the keystore */

		const msg = 'hello world';

		const ciphertext = await keystore.encrypt(msg, encryptionKeySpki, passKeySalt);
		const plaintext = await keystore.decrypt(ciphertext, encryptionKeySpki, passKeySalt);
		if (plaintext !== msg) {
			setError('Keystore is invalid: ' + plaintext + ' != ' + msg);
			throw new Error('Keystore is invalid: ' + plaintext + ' != ' + msg);
		}
		const signature = await keystore.sign(msg);
		const verified = await keystore.verify(msg, signature, apiKeySpki); 
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
				purgeKeystore,
			}}
		>
			{children}
		</KeystoreContext.Provider>
	);
};

export const useKeystore = () => useContext(KeystoreContext);
