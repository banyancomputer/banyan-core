export type Msg = ArrayBuffer | string | Uint8Array

export type CipherText = ArrayBuffer
export type SymmKey = CryptoKey
export type SymmWrappingKey = CryptoKey

export type PublicKey = CryptoKey
export type PrivateKey = CryptoKey

export type Config = {
  // Asymmetric Configuration
  exchangeAlg: string
  writeAlg: string
  curve: EccCurve

  // Symmetric Configuration
  symmAlg: SymmAlg
  symmKeyLength: SymmKeyLength
  // derivedBitLength: DerivedBitLength
  
  // Hash Configuration
  hashAlg: HashAlg
  charSize: CharSize
  
  // Key Store Configuration
  storeName: string
}

export type SymmKeyOpts = {
  alg: SymmAlg
  length: SymmKeyLength
  iv: ArrayBuffer
}

export enum ExportKeyFormat {
  PKCS8 = 'pkcs8',
  SPKI = 'spki',
  RAW = 'raw',
}

export enum CryptoSystem {
  ECC = 'ecc',
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
export interface KeyMaterial {
  encryptionKeyPair: CryptoKeyPair,
  apiKeyPair: CryptoKeyPair,
}

// Escrowed device key material stored on the platform
export interface EscrowedKeyMaterial {
  encryptionKeyPem: string,
  apiKeyPem: string,
  wrappedEncryptionKey: string,
  wrappedApiKey: string,
  passKeySalt: string,
}

// Exported private key material that can be used by rust
export interface ExportedKeyMaterial {
  // Pkcs8 encoded private key pem
  encryptionKeyPem: string,
  // Pkcs8 encoded private key pem
  apiKeyPem: string,
}

// Format of cached key material blob placed in user's browser storage
export interface CachedKeyMaterial {
  // The encrypted key material
  encryptedExportedKeyMaterial: string,
  // The salt needed to derive the session key
  salt: string,
}

export interface KeyStore {
  cfg: Config

  /* Keystore Management */

  blobExists(blobName: string): Promise<boolean>
  deleteBlob(blobName: string): Promise<void>
  destroy(): Promise<void>

  // Cache exported key material associated with the current session
  cacheKeyMaterial(
    keyMaterial: ExportedKeyMaterial,
    sessionKey: string,
    sessionId: string,
    cfg?: Partial<Config>
  ): Promise<void>

  // Retrieve cached key material associated with the current session
  retrieveCachedKeyMaterial(
    sessionKey: string,
    sessionId: string,
    cfg?: Partial<Config>
  ): Promise<ExportedKeyMaterial>
}
