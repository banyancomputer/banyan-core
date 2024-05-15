export type Msg = ArrayBuffer | string | Uint8Array;

export type CipherText = ArrayBuffer;
export type SymmKey = CryptoKey;

export type PublicKey = CryptoKey;
export type PrivateKey = CryptoKey;

export type Config = {
    // Asymmetric Configuration
    exchangeAlg: string;
    writeAlg: string;
    curve: EccCurve;

    // Symmetric Configuration
    symmAlg: SymmAlg;
    symmKeyLength: SymmKeyLength;
    // derivedBitLength: DerivedBitLength

    // Hash Configuration
    hashAlg: HashAlg;
    charSize: CharSize;

    // Key Store Configuration
    storeName: string;
};

export type SymmKeyOpts = {
    alg: SymmAlg;
    length: SymmKeyLength;
    iv: ArrayBuffer;
};

export enum ExportKeyFormat {
    PKCS8 = 'pkcs8',
    SPKI = 'spki',
    RAW = 'raw',
}

export enum EccCurve {
    P_384 = 'P-384',
}

export enum SymmAlg {
    AES_GCM = 'AES-GCM',
    AES_KW = 'AES-KW',
}

export enum SymmKeyLength {
    B256 = 256,
    B384 = 384,
    B512 = 512,
}

export enum HashAlg {
    SHA_256 = 'SHA-256',
    SHA_384 = 'SHA-384',
    SHA_512 = 'SHA-512',
}

export enum CharSize {
    B8 = 8,
    B16 = 16,
}

export enum KeyUse {
    Exchange = 'exchange',
    Write = 'write',
}

// Generated device key material on platform sign-up
// This is the only time CryptoKeyPairs are generated,
// after which everything is PEM formatted Strings
export interface KeyMaterial {
    pair: CryptoKeyPair;
}

// Escrowed device key material stored on the platform
export interface EscrowedKeyMaterial {
    // SPKI encoded public key pem
    publicKey: string;
    // AES encrypted instance of Json.stringify(PrivateKeyMaterial)
    encryptedPrivateKeyMaterial: string;
    // Salt used to derive the decryption key
    passKeySalt: string;
}

// Exported private key material that can be used by rust
export interface PrivateKeyMaterial {
    // Pkcs8 encoded private key pem
    privateKeyPem: string;
}

// Format of cached key material blob placed in user's browser storage
export interface CachedKeyMaterial {
    // AES encrypted instance of Json.stringify(PrivateKeyMaterial)
    encryptedPrivateKeyMaterial: string;
    // The salt needed to derive the session key
    sessionKeySalt: string;
}

export interface KeyStore {
    cfg: Config;

    /* Keystore Management */

    blobExists(blobName: string): Promise<boolean>;
    deleteBlob(blobName: string): Promise<void>;
    destroy(): Promise<void>;

    // Cache exported key material associated with the current session
    cachePrivateKeyMaterial(
        privateKeyMaterial: PrivateKeyMaterial,
        sessionKey: string,
        sessionId: string,
        cfg?: Partial<Config>
    ): Promise<void>;

    // Retrieve cached key material associated with the current session
    retrieveCachedPrivateKeyMaterial(
        sessionKey: string,
        sessionId: string,
        cfg?: Partial<Config>
    ): Promise<PrivateKeyMaterial>;
}
