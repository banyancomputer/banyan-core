import { createContext, useState, useEffect, useContext } from 'react';
import {
	DevicePublicKey,
	EscrowedDeviceKeyPair,
	EscrowedDevicePrivateKey,
} from '@/lib/db/entities';
import { ClientApi } from '@/lib/api/auth';
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
	const { data: session } = useSession();

	// External State
	const [isRegistered, setIsRegistered] = useState<boolean>(false);
	const [keystoreInitialized, setKeystoreInitialized] =
		useState<boolean>(false);

	// Internal State
	const api = new ClientApi();
	const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
	const [escrowedDeviceKeyPair, setEscrowedDeviceKeyPair] =
		useState<EscrowedDeviceKeyPair | null>(null);
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
			const ks = await getKeystore(session.userId);
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
			const resp = await api.readEscrowedKeyPair().catch((err) => {
				return undefined;
			});
			if (resp) {
				setIsRegistered(true);
				setEscrowedDeviceKeyPair(resp.escrowed_device_key_pair);
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
		if (escrowedDeviceKeyPair && !keystoreInitialized) {
			console.log('Initializing keystore with recovered escrowed data');
			await initKeystore(session, escrowedDeviceKeyPair, passkey);
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
		const user_id: string = session.userId;
		// Get the keystore for the user from the browser
		const ks = await getKeystore(user_id);
		// Generate a new keypairs for the user
		await ks.genExchangeKeyPair();
		await ks.genWriteKeyPair();
		// Derive a new passkey for the user -- this generates a random salt
		const passkey_salt: string = await ks.deriveEscrowKey(passphrase);
		const wrapped_ecdh_key_pair = await ks.exportEscrowedExchangeKeyPair();
		const wrapped_ecdsa_key_pair = await ks.exportEscrowedWriteKeyPair();

		// Assoicate the public key in the db with the user
		const devicePublicKey: Partial<DevicePublicKey> = {
			ecdsa_fingerprint: await ks.fingerprintPublicWriteKey(),
			ecdh_spki_pem: pemWrap(wrapped_ecdh_key_pair.publicKeyStr),
			ecdsa_spki_pem: pemWrap(wrapped_ecdsa_key_pair.publicKeyStr),
		};

		// Create the user in the db with a reference to the pubkey and the encrypted private key
		const escrowedDevicePrivateKey: Partial<EscrowedDevicePrivateKey> = {
			device_public_key_ecdsa_fingerprint: devicePublicKey.ecdsa_fingerprint,
			wrapped_ecdsa_pkcs8: wrapped_ecdsa_key_pair.wrappedPrivateKeyStr,
			wrapped_ecdh_pkcs8: wrapped_ecdh_key_pair.wrappedPrivateKeyStr,
			passkey_salt: passkey_salt,
		};

		await api.escrowKeyPair({
			device_public_key: devicePublicKey,
			escrowed_device_private_key: escrowedDevicePrivateKey,
		});
	};

	const initKeystore = async (
		session: Session,
		escrowedDeviceKeyPair: EscrowedDeviceKeyPair,
		passphrase: string
	) => {
		// Get the keystore by the user's uid
		const ks = await getKeystore(session.userId);

		// Check if the user's keystore is already initialized
		if (
			(await ks.keyExists(ESCROW_KEY_NAME)) &&
			(await ks.keyPairExists(EXCHANGE_KEY_PAIR_NAME)) &&
			(await ks.keyPairExists(WRITE_KEY_PAIR_NAME))
		) {
			return;
		}

		// Read the user's encrypted private key and salt and derive the passkey
		const { device_public_key, escrowed_device_private_key } =
			escrowedDeviceKeyPair;
		const { ecdsa_fingerprint, ecdh_spki_pem, ecdsa_spki_pem } =
			device_public_key;
		const { wrapped_ecdsa_pkcs8, wrapped_ecdh_pkcs8, passkey_salt } =
			escrowed_device_private_key;

		await ks.deriveEscrowKey(passphrase, passkey_salt);

		// Import the keypair into the keystore
		await ks.importEscrowedWriteKeyPair({
			publicKeyStr: pemUnwrap(ecdsa_spki_pem),
			wrappedPrivateKeyStr: wrapped_ecdsa_pkcs8,
		});
		await ks.importEscrowedExchangeKeyPair({
			publicKeyStr: pemUnwrap(ecdh_spki_pem),
			wrappedPrivateKeyStr: wrapped_ecdh_pkcs8,
		});

		// Check that the keystore is valid

		// Check that the fingerprint matches
		const fingerprint = await ks.fingerprintPublicWriteKey();
		if (fingerprint !== ecdsa_fingerprint) {
			setError(
				'Keystore is invalid: ' + fingerprint + ' != ' + ecdsa_fingerprint
			);
			throw new Error(
				'Keystore is invalid: ' + fingerprint + ' != ' + ecdsa_fingerprint
			);
		}

		const msg = 'hello world';

		const ciphertext = await ks.encrypt(
			msg,
			pemUnwrap(ecdh_spki_pem),
			passkey_salt
		);
		const plaintext = await ks.decrypt(
			ciphertext,
			pemUnwrap(ecdh_spki_pem),
			passkey_salt
		);
		if (plaintext !== msg) {
			setError('Keystore is invalid: ' + plaintext + ' != ' + msg);
			throw new Error('Keystore is invalid: ' + plaintext + ' != ' + msg);
		}
		const signature = await ks.sign(msg);
		const verified = await ks.verify(msg, signature, pemUnwrap(ecdsa_spki_pem));
		if (!verified) {
			setError('Keystore is invalid (signature)');
			throw new Error('Keystore is invalid (signature)');
		}
	};

	const pemWrap = (spki: string) => {
		// Wrap the public key in a pem
		const pemHeader = '-----BEGIN PUBLIC KEY-----\n';
		const pemFooter = '\n-----END PUBLIC KEY-----';
		const pem = pemHeader + spki + pemFooter;
		return pem;
	};

	const pemUnwrap = (pem: string) => {
		// Unwrap the public key from a pem
		const pemHeader = '-----BEGIN PUBLIC KEY-----';
		const pemFooter = '-----END PUBLIC KEY-----';
		const spki = pem.replace(pemHeader, '').replace(pemFooter, '');
		return spki;
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
