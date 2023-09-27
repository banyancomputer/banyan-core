import { createContext, useContext, useEffect, useState } from 'react';
import ECCKeystore from '@/lib/crypto/ecc/keystore';
import { importKeyPair } from '@/lib/crypto/ecc';
import {
    fingerprintEcPem,
    prettyFingerprint,
} from '@/lib/crypto/utils';
import { useSession } from 'next-auth/react';
import { DeviceApiKey } from '@/lib/interfaces';
import { ClientApi } from '@/lib/api/auth';
import {
    publicPemUnwrap,
    privatePemUnwrap,
} from '@/utils';
import { EscrowedKeyMaterial, KeyUse, EccCurve, ExportedKeyMaterial } from '@/lib/crypto/types';
import { setCookie, destroyCookie, parseCookies } from 'nookies';

const KEY_STORE_NAME_PREFIX = 'banyan-key-cache';
const KEY_STORE_COOKIE_NAME = 'banyan-key-cookie';

interface SessionKey {
    sessionId: string;
    sessionKey: string;
}

// These can be short because they're only used to cache the key material
const KEY_STORE_COOKIE_MAX_AGE = 60 * 60 * 24 * 7 * 4 * 3; // 3 months

const getSessionKey = (): SessionKey => {
    const cookies = parseCookies();
    // Try and get the session key from cookies
    // If DNE or Expired, create a new one
    if (!cookies[KEY_STORE_COOKIE_NAME]) {
        return setSessionKeyCookie();
    }
    const [sessionId, sessionKey] = cookies[KEY_STORE_COOKIE_NAME].split(':');
    return { sessionId, sessionKey };
}

const setSessionKeyCookie = (): SessionKey => {
    const sessionId = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    const sessionKey = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    setCookie(null, KEY_STORE_COOKIE_NAME, `${sessionId}:${sessionKey}`, {
        // TODO: Are there any security implications to setting this to lax?
        maxAge: KEY_STORE_COOKIE_MAX_AGE,
        lax: process.env.NODE_ENV === 'development',
        sameSite: 'strict',
        path: '/',
    });
    return { sessionId, sessionKey };
}

export const KeystoreContext = createContext<{
    // External State

    // Whether the user's keystore has been initialized
    keystoreInitialized: boolean;

    // External Methods

    // Initialize a keystore based on the user's passphrase
    initializeKeystore: (passkey: string) => Promise<void>;
    // Get the user's Encryption Key Pair
    getEncryptionKey: () => Promise<CryptoKeyPair>;
    // Get the user's API Key Pair
    getApiKey: () => Promise<CryptoKeyPair>;
    // Purge the keystore from storage
    purgeKeystore: () => Promise<void>;
    isLoading: boolean,
    escrowedDevice: EscrowedKeyMaterial | null;
}>({
    keystoreInitialized: false,
    getEncryptionKey: async () => {
        throw new Error('Keystore not initialized');
    },
    getApiKey: async () => {
        throw new Error('Keystore not initialized');
    },
    initializeKeystore: async (passkey: string) => { },
    purgeKeystore: async () => { },
    isLoading: false,
    escrowedDevice: null
});

export const KeystoreProvider = ({ children }: any) => {
    /* State */
    const { data: session } = useSession();

    // External State
    const [keystoreInitialized, setKeystoreInitialized] = useState<boolean>(false);
    const [isLoading, setIsLoading] = useState<boolean>(true);

    // Internal State
    const api = new ClientApi();
    const [keystore, setKeystore] = useState<ECCKeystore | null>(null);
    const [escrowedDevice, setEscrowedDevice] = useState<EscrowedKeyMaterial | null>(null);
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
                let sessionKey = getSessionKey();
                try {
                    await ks.retrieveCachedKeyMaterial(
                        sessionKey.sessionKey, sessionKey.sessionId
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
        if (session) {
            setEscrowedDevice(session.escrowedKeyMaterial);
            !session.escrowedKeyMaterial && purgeKeystore();
        }
    }, [session]);

    /* Methods */

    // Initialize a keystore based on the user's passphrase
    const initializeKeystore = async (passkey: string): Promise<void> => {
        let exportedKeyMaterial: ExportedKeyMaterial;
        // TODO: better error handling
        if (!keystore) {
            setError('No keystore');
            throw new Error('Keystore not initialized');
        }
        try {
            if (escrowedDevice) {
                exportedKeyMaterial = await recoverDevice(passkey);
            } else {
                exportedKeyMaterial = await escrowDevice(passkey);
            }
            let sessionKey = getSessionKey();
            // Cache the key material encrypted with the session key
            await keystore.cacheKeyMaterial(
                exportedKeyMaterial,
                sessionKey.sessionKey,
                sessionKey.sessionId
            );
            setKeystoreInitialized(true);
        } catch (err: any) {
            console.error(err);
            setError("Error initializing keystore: " + err.message);
            throw new Error(err.message);
        }
    };

    // TODO: Just return the key material eventually
    // Get the user's Encryption Key Pair as a CryptoKeyPair
    const getEncryptionKey = async (): Promise<CryptoKeyPair> => {
        // TODO: better error handling
        if (!keystore) {
            setError('No keystore');
            throw new Error('No keystore');
        }
        if (!keystoreInitialized) {
            setError('Keystore not initialized');
            throw new Error('Keystore not initialized');
        }
        if (!escrowedDevice) {
            setError('Missing escrowed data');
            throw new Error('Missing escrowed data');
        }
        let sessionKey = getSessionKey();
        const exportKeyMaterial = await keystore.retrieveCachedKeyMaterial(
            sessionKey.sessionKey, sessionKey.sessionId
        );
        const encryptionPublicKeyPem = escrowedDevice.encryptionKeyPem;
        const encryptionPublicKeySpki = publicPemUnwrap(encryptionPublicKeyPem);
        const encryptionPrivateKeyPem = exportKeyMaterial.encryptionKeyPem;
        const encryptionPrivateKeyPkcs8 = privatePemUnwrap(encryptionPrivateKeyPem);
        // Assume P-384 for now 
        const encryptionKeyPair = await importKeyPair(
            encryptionPublicKeySpki,
            encryptionPrivateKeyPkcs8,
            EccCurve.P_384,
            KeyUse.Exchange
        );
        return encryptionKeyPair;
    };

    // TODO: Just return the key material eventually
    // Get the user's API Key as a CryptoKeyPair
    const getApiKey = async (): Promise<CryptoKeyPair> => {
        // TODO: better error handling
        if (!keystore || !keystoreInitialized) {
            setError('Keystore not initialized');
            throw new Error('Keystore not initialized');
        }
        if (!escrowedDevice) {
            setError('Missing escrowed data');
            throw new Error('Missing escrowed data');
        }
        let sessionKey = getSessionKey();
        const exportKeyMaterial = await keystore.retrieveCachedKeyMaterial(
            sessionKey.sessionKey, sessionKey.sessionId
        );
        const apiKeyPublicKeyPem = escrowedDevice.apiKeyPem;
        const apiKeyPublicKeySpki = publicPemUnwrap(apiKeyPublicKeyPem);
        const apiKeyPrivateKeyPem = exportKeyMaterial.apiKeyPem;
        const apiKeyPrivateKeyPkcs8 = privatePemUnwrap(apiKeyPrivateKeyPem);
        // Assume P-384 for now      
        const apiKeyPair = await importKeyPair(
            apiKeyPublicKeySpki,
            apiKeyPrivateKeyPkcs8,
            EccCurve.P_384,
            KeyUse.Write
        );
        return apiKeyPair;
    };

    // Purge the keystore from storage
    const purgeKeystore = async (): Promise<void> => {
        if (!keystore) {
            setError('No keystore');
            throw new Error('No keystore');
        }
        await keystore.clear();
        // Purge the session key cookie
        destroyCookie(null, KEY_STORE_COOKIE_NAME);
        setKeystoreInitialized(false);
    };

    /* Helpers */

    // Register a new user in firestore
    const escrowDevice = async (passphrase: string): Promise<ExportedKeyMaterial> => {
        if (!keystore) {
            setError('No keystore');
            throw new Error('No keystore');
        }
        const keyMaterial = await keystore.genKeyMaterial();
        const escrowedKeyMaterial = await keystore.escrowKeyMaterial(
            keyMaterial,
            passphrase
        );
        setEscrowedDevice(escrowedKeyMaterial);
        const exportedKeyMaterial = await keystore.exportKeyMaterial(keyMaterial);

        // Escrow the user's private key material
        await api
            .escrowDevice(escrowedKeyMaterial)
            .then((resp) => {
                setEscrowedDevice(resp);
            })
            .catch((err) => {
                setError("Error escrowing device: " + err.message);
                throw new Error("Error escrowing device: " + err.message);
            });

        const apiKeyFingerprint = await fingerprintEcPem(
            escrowedKeyMaterial.apiKeyPem,
            EccCurve.P_384,
            KeyUse.Write
        ).then(prettyFingerprint).catch((err) => {
            setError('Error fingerprinting API key: ' + err.message);
            throw new Error('Error fingerprinting API key: ' + err.message);
        });

        // Register the user's public key material
        await api
            .registerDeviceApiKey(escrowedKeyMaterial.apiKeyPem)
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
        return exportedKeyMaterial;
    };

    // Recovers a device's private key material from escrow
    const recoverDevice = async (passphrase: string) => {
        if (!keystore) {
            setError('No keystore');
            throw new Error('No keystore');
        }
        if (!escrowedDevice) {
            setError('No escrowed device');
            throw new Error('No escrowed device');
        }
        return await keystore.recoverKeyMaterial(
            escrowedDevice,
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
                escrowedDevice
            }}
        >
            {children}
        </KeystoreContext.Provider>
    );
};

export const useKeystore = () => useContext(KeystoreContext);
