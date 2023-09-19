import config from '../config';
import IDB from '../idb';
import { ECCNotEnabled } from '../errors';
import * as ecc from './index';
import utils from '../utils';
import KeyStoreBase from '../keystore/base';
import {
  Config,
  KeyStore,
  PublicKey,
  PrivateKey,
  KeyUse,
  KeyMaterial,
  EscrowedKeyMaterial,
  ExportedKeyMaterial
} from '../types';
import { DEFAULT_SALT_LENGTH } from '../constants';
import pbkdf2 from '../pbkdf2/index';
import { publicPemWrap, publicPemUnwrap, privatePemWrap } from '@/utils/pem';

export default class ECCKeyStore extends KeyStoreBase implements KeyStore {
  static async init(maybeCfg: Partial<Config>): Promise<ECCKeyStore> {
    const eccEnabled = await config.eccEnabled();
    if (!eccEnabled) {
      throw ECCNotEnabled;
    }
    const cfg = config.normalize({
      ...(maybeCfg || {}),
    });
    const { storeName } = cfg;
    const store = IDB.createStore(storeName);
    return new ECCKeyStore(cfg, store);
  }

  // Key Pair Generation

  // Generate Key Material and return to the caller
  async genKeyMaterial(cfg?: Partial<Config>): Promise<KeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const encryptionKeyPair = await ecc.genKeyPair(mergedCfg.curve, KeyUse.Exchange);
    const apiKeyPair = await ecc.genKeyPair(mergedCfg.curve, KeyUse.Write);
    return {
      encryptionKeyPair,
      apiKeyPair
    } as KeyMaterial;
  }
  
  // Escrow Stuff

  async exportKeyMaterial(keyMaterial: KeyMaterial, cfg?: Partial<Config>): Promise<ExportedKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const encryptionKeyPem = await ecc.exportPrivateKey(keyMaterial.encryptionKeyPair.privateKey as PrivateKey);
    const apiKeyPem = await ecc.exportPrivateKey(keyMaterial.apiKeyPair.privateKey as PrivateKey);
    return {
      encryptionKeyPem,
      apiKeyPem
    } as ExportedKeyMaterial;
  }

  // Escrow Key Material and return to the caller
  // Performs first-time escrow of the key material
  async escrowKeyMaterial(keyMaterial: KeyMaterial, passphrase: string, cfg?: Partial<Config>): Promise<EscrowedKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const salt = utils.randomBuf(DEFAULT_SALT_LENGTH);
    const key = await pbkdf2.deriveKey(
      passphrase,
      salt,
      mergedCfg.hashAlg,
      ['wrapKey', 'unwrapKey'],
      config.symmKeyOpts(mergedCfg)
    );
    // Get the public key pems from the key material
    const encryptionKey = await ecc.exportPublicKey(keyMaterial.encryptionKeyPair.publicKey as PublicKey);
    const encryptionKeyPem = publicPemWrap(encryptionKey);
    const apiKey = await ecc.exportPublicKey(keyMaterial.apiKeyPair.publicKey as PublicKey);
    const apiKeyPem = publicPemWrap(apiKey);
    // Get the wrapped private keys from the key material
    const wrappedEncryptionKey = await ecc.exportEscrowedPrivateKey(
      keyMaterial.encryptionKeyPair.privateKey as PrivateKey,
      key
    );
    const wrappedApiKey = await ecc.exportEscrowedPrivateKey(
      keyMaterial.apiKeyPair.privateKey as PrivateKey,
      key
    );
    // Return the escrowed key material
    return {
      encryptionKeyPem,
      apiKeyPem,
      wrappedEncryptionKey,
      wrappedApiKey,
      passKeySalt: utils.arrBufToBase64(salt)
    } as EscrowedKeyMaterial;
  }

  // Recover Key Material from escrowed key material and return to the caller
  // Performs recovery of the key material from the platform
  async recoverKeyMaterial(escrowedKeyMaterial: EscrowedKeyMaterial, passphrase: string, cfg?: Partial<Config>): Promise<ExportedKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const salt = utils.base64ToArrBuf(escrowedKeyMaterial.passKeySalt);
    const key = await pbkdf2.deriveKey(
      passphrase,
      salt,
      mergedCfg.hashAlg,
      ['wrapKey', 'unwrapKey'],
      config.symmKeyOpts(mergedCfg)
    );
    // Import the key pairs from the escrowed key material
    const encryptionPublicKey = publicPemUnwrap(escrowedKeyMaterial.encryptionKeyPem);
    const encryptionKeyPair = await ecc.importEscrowedKeyPair(
      encryptionPublicKey,
      escrowedKeyMaterial.wrappedEncryptionKey,
      key,
      mergedCfg.curve,
      KeyUse.Exchange
    );
    const apiPublicKey = publicPemUnwrap(escrowedKeyMaterial.apiKeyPem);
    const apiKeyPair = await ecc.importEscrowedKeyPair(
      apiPublicKey,
      escrowedKeyMaterial.wrappedApiKey,
      key,
      mergedCfg.curve,
      KeyUse.Write
    );
    // Export the private key pems from the key material
    const encryptionKey= await ecc.exportPrivateKey(encryptionKeyPair.privateKey as PrivateKey);
    const apiKey= await ecc.exportPrivateKey(apiKeyPair.privateKey as PrivateKey);
    const encryptionKeyPem = privatePemWrap(encryptionKey);
    const apiKeyPem = privatePemWrap(apiKey);
    // Return the exported key material
    return {
      encryptionKeyPem,
      apiKeyPem
    } as ExportedKeyMaterial;
  }
}
