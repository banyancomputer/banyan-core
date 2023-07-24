import { createContext, useState, useEffect, useContext } from 'react';
import { IEscrowedKey, IPublicKey } from '@/lib/db/entities';
import { EscrowedKeyApi, PublicKeyApi } from '@/lib/client/api';
import ECCKeystore from 'banyan-webcrypto-experiment/ecc/keystore';
import { clear as clearIdb } from 'banyan-webcrypto-experiment/idb';
import { useSession } from 'next-auth/react';
import { Session } from 'next-auth';

const KEY_STORE_NAME_PREFIX = 'key-store';
const EXCHANGE_KEY_PAIR_NAME = 'exchange-key-pair';
const WRITE_KEY_PAIR_NAME = 'write-key-pair';
const ESCROW_KEY_NAME = 'escrow-key';

export const KeystoreContext = createContext<{
	// External State

	// Whether the user has an encrypted private key in the db
	isRegistered: boolean;
	// Whether the user's keystore has been initialized
	keystoreInitialized: boolean;
	
	// External Methods

	// Initialize a keystore based on the user's passphrase
	initializeKeystore: (session: Session, passkey: string) => Promise<void>;
	// Purge the keystore from storage
	purgeKeystore: () => Promise<void>;
	// Get the public key's fingerprint
	getFingerprint: () => Promise<string>;
}>({
	isRegistered: false,
	keystoreInitialized: false,
	initializeKeystore: async (session: Session, passkey: string) => {},
	purgeKeystore: async () => {},
	getFingerprint: async () => '',
});

export const KeystoreProvider = ({ children }: any) => {
	/* State */

	// Inherited State
	const { data: session } = useSession();

	// External State
	const [isRegistered, setIsRegistered] = useState<boolean>(false);
	const [keystoreInitialized, setKeystoreInitialized] =
		useState<boolean>(false);

	// Internal State
	const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
	const [escrowedKey, setEscrowedKey] = useState<IEscrowedKey | null>(null);
	const [error, setError] = useState<string | null>(null);

	/* Effects */

	// Handle errors
	useEffect(() => {
		if (error) {
			console.error(error);
		}
	}, [error]);

	// Attempt to initialize the keystore when the session changes
	useEffect(() => {
		const tryInitKeystore = async (session: Session) => {
			const ks = await getKeystore(session.id);
			if (
				(await ks.keyExists(ESCROW_KEY_NAME)) &&
				(await ks.keyPairExists(EXCHANGE_KEY_PAIR_NAME)) &&
				(await ks.keyPairExists(WRITE_KEY_PAIR_NAME))
			) {
				setKeystore(ks);
				setKeystoreInitialized(true);
			}
		};
		if (session) {
			tryInitKeystore(session);
		}
	}, [session]);

	// Set the isRegistered state if the user has an encrypted private key in the db
	useEffect(() => {
		const check = async (session: Session) => {
			const escrowedKey = await EscrowedKeyApi.read().catch((err) => {
				return undefined;
			});
			if (escrowedKey) {
				setIsRegistered(true);
				setEscrowedKey(escrowedKey);
			}
		};
		if (session) {
			check(session);
		}
	}, [session]);

	

	/* Methods */

	// Initialize a keystore based on the user's passphrase
	const initializeKeystore = async (
		session: Session,
		passkey: string
	): Promise<void> => {
		console.log('Initializing keystore');
		if (escrowedKey && !keystoreInitialized) {
			console.log('Initializing keystore with recovered escrowed data');
			await initKeystore(session, escrowedKey, passkey);
		} else {
			console.log('Registering user');
			await registerUser(session, passkey);
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
		setIsRegistered(false);
		setKeystoreInitialized(false);
	};

	/* Helpers */

	// Initialize a keystore pointed by the user's uid
	const getKeystore = async (uid: string) => {
		const storeName = KEY_STORE_NAME_PREFIX + '-' + uid;
		if (keystore) {
			return keystore;
		}
		// Defaults are fine here
		const ks = await ECCKeystore.init({ storeName });
		setKeystore(ks);
		return ks;
	};

	// Register a new user in firestore
	const registerUser = async (
		session: Session,
		passphrase: string
	): Promise<void> => {
		// Get the uid of the new user
		const owner: string = session.id;
		// Get the keystore for the user from the browser
		const ks = await getKeystore(owner);
		// Generate a new keypairs for the user
		await ks.genExchangeKeyPair();
		await ks.genWriteKeyPair();
		// Derive a new passkey for the user -- this generates a random salt
		const passkey_salt: string = await ks.deriveEscrowKey(passphrase);
		const wrapped_ecdh_key_pair = await ks.exportEscrowedExchangeKeyPair();
		const wrapped_ecdsa_key_pair = await ks.exportEscrowedWriteKeyPair();

		// Assoicate the public key in the db with the user
		const publicKey: IPublicKey = {
			ecdsa_fingerprint: await ks.fingerprintPublicWriteKey(),
			ecdh_spki: wrapped_ecdh_key_pair.publicKeyStr,
			ecdsa_spki: wrapped_ecdsa_key_pair.publicKeyStr,
			owner: owner,
		};
		await PublicKeyApi.create(publicKey).catch((err) => {
			setError('Failed to create public key: ' + err.message);
			console.error(err);
			return undefined;
		});

		// Create the user in the db with a reference to the pubkey and the encrypted private key
		const escrowedKey: IEscrowedKey = {
			ecdsa_pubkey_fingerprint: publicKey.ecdsa_fingerprint,
			wrapped_ecdsa_pkcs8: wrapped_ecdsa_key_pair.wrappedPrivateKeyStr,
			wrapped_ecdh_pkcs8: wrapped_ecdh_key_pair.wrappedPrivateKeyStr,
			passkey_salt: passkey_salt,
			owner: owner,
		};
		await EscrowedKeyApi.create(escrowedKey).catch((err) => {
			setError('Failed to create escrowed key: ' + err.message);
			console.error(err);
			return undefined;
		});
	};

	const initKeystore = async (
		session: Session,
		escrowedKey: IEscrowedKey,
		passphrase: string
	) => {
		// Get the keystore by the user's uid
		const ks = await getKeystore(session.id);

		// Check if the user's keystore is already initialized
		if (
			(await ks.keyExists(ESCROW_KEY_NAME)) &&
			(await ks.keyPairExists(EXCHANGE_KEY_PAIR_NAME)) &&
			(await ks.keyPairExists(WRITE_KEY_PAIR_NAME))
		) {
			return;
		}

		// Read the user's encrypted private key and salt and derive the passkey
		const {
			ecdsa_pubkey_fingerprint,
			wrapped_ecdsa_pkcs8,
			wrapped_ecdh_pkcs8,
			passkey_salt,
		} = escrowedKey;

		await ks.deriveEscrowKey(passphrase, passkey_salt);
		const publicKey = await PublicKeyApi.read(ecdsa_pubkey_fingerprint);
		const { ecdh_spki, ecdsa_spki } = publicKey;

		// Import the keypair into the keystore
		await ks.importEscrowedWriteKeyPair({
			publicKeyStr: ecdsa_spki,
			wrappedPrivateKeyStr: wrapped_ecdsa_pkcs8,
		});
		await ks.importEscrowedExchangeKeyPair({
			publicKeyStr: ecdh_spki,
			wrappedPrivateKeyStr: wrapped_ecdh_pkcs8,
		});

		// Check that the keystore is valid
		const msg = 'hello world';

		const ciphertext = await ks.encrypt(msg, ecdh_spki, passkey_salt);
		const plaintext = await ks.decrypt(ciphertext, ecdh_spki, passkey_salt);
		if (plaintext !== msg) {
			setError('Keystore is invalid: ' + plaintext + ' != ' + msg);
			throw new Error('Keystore is invalid: ' + plaintext + ' != ' + msg);
		}
		const signature = await ks.sign(msg);
		const verified = await ks.verify(msg, signature, ecdsa_spki);
		if (!verified) {
			setError('Keystore is invalid (signature)');
			throw new Error('Keystore is invalid (signature)');
		}
	};

	return (
		<KeystoreContext.Provider
			value={{
				isRegistered,
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
