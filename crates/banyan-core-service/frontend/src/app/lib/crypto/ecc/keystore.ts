import config from '../config';
import IDB from '../idb';
import { ECCNotEnabled } from '../errors';
import * as ecc from './index';
import utils from '../utils';
import KeyStoreBase from '../keystore/base';
import aes from '../aes/index';
import {
  Config,
  KeyStore,
  PublicKey,
  PrivateKey,
  KeyUse,
  KeyMaterial,
  EscrowedKeyMaterial,
  PrivateKeyMaterial
} from '../types';
import { DEFAULT_SALT_LENGTH } from '../constants';
import pbkdf2 from '../pbkdf2/index';

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
  
  async exportPrivateKeyMaterial(keyMaterial: KeyMaterial, cfg?: Partial<Config>): Promise<PrivateKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const encryptionPrivateKeyPem = await ecc.exportPrivateKeyPem(keyMaterial.encryptionKeyPair.privateKey as PrivateKey);
    const apiPrivateKeyPem = await ecc.exportPrivateKeyPem(keyMaterial.apiKeyPair.privateKey as PrivateKey);
    return {
      encryptionPrivateKeyPem,
      apiPrivateKeyPem
    } as PrivateKeyMaterial;
  }

  // Escrow Key Material and return to the caller
  // Performs first-time escrow of the key material
  async escrowKeyMaterial(keyMaterial: KeyMaterial, passphrase: string, cfg?: Partial<Config>): Promise<EscrowedKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const salt = utils.randomBuf(DEFAULT_SALT_LENGTH);

    // Get the public key pems from the key material
    const encryptionPublicKeyPem = await ecc.exportPublicKeyPem(keyMaterial.encryptionKeyPair.publicKey as PublicKey);
    const apiPublicKeyPem = await ecc.exportPublicKeyPem(keyMaterial.apiKeyPair.publicKey as PublicKey);

    // Get the PrivateKeyMaterial from the key material
    const encryptionPrivateKeyPem = await ecc.exportPrivateKeyPem(keyMaterial.encryptionKeyPair.privateKey as PrivateKey);
    const apiPrivateKeyPem = await ecc.exportPrivateKeyPem(keyMaterial.apiKeyPair.privateKey as PrivateKey);
    let privateKeyMaterial = {
      encryptionPrivateKeyPem,
      apiPrivateKeyPem
    } as PrivateKeyMaterial;

    // Derive a key from the passphrase and salt
    const key = await pbkdf2.deriveKey(
      passphrase,
      salt,
      mergedCfg.hashAlg,
      ['encrypt'],
      config.symmKeyOpts(mergedCfg)
    );

    const privateKeyMaterialString = JSON.stringify(privateKeyMaterial);
    const encryptedPrivateKeyMaterial = await aes.encrypt(
      privateKeyMaterialString,
      key,
    );
    
    // Return the escrowed key material
    return {
      encryptionPublicKeyPem,
      apiPublicKeyPem,
      encryptedPrivateKeyMaterial,
      passKeySalt: utils.arrBufToBase64(salt),
    } as EscrowedKeyMaterial;
  }

  // Recover Key Material from escrowed key material and return to the caller
  // Performs recovery of the key material from the platform
  async recoverKeyMaterial(escrowedKeyMaterial: EscrowedKeyMaterial, passphrase: string, cfg?: Partial<Config>): Promise<PrivateKeyMaterial> {
    const mergedCfg = config.merge(this.cfg, cfg);
    const salt = utils.base64ToArrBuf(escrowedKeyMaterial.passKeySalt);
    const key = await pbkdf2.deriveKey(
      passphrase,
      salt,
      mergedCfg.hashAlg,
      ['decrypt'],
      config.symmKeyOpts(mergedCfg)
    );
    
    let privateKeyMaterialString = await aes.decrypt(
      escrowedKeyMaterial.encryptedPrivateKeyMaterial,
      key
    );
    let privateKeyMaterial = JSON.parse(privateKeyMaterialString);
    return privateKeyMaterial as PrivateKeyMaterial;
  }
}
