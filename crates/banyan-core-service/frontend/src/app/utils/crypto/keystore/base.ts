import aes from '../aes';
import pbkdf2 from '../pbkdf2/index';
import idb from '../idb';
import utils from '../utils';
import config from '../config';
import {
    CachedKeyMaterial,
    Config,
    PrivateKeyMaterial,
} from '../types';
import { DEFAULT_SALT_LENGTH } from '../constants';

export default class KeyStoreBase {
    cfg: Config;
    protected store: LocalForage;

    constructor(cfg: Config, store: LocalForage) {
        this.cfg = cfg;
        this.store = store;
    }

    static async initBase(maybeCfg?: Partial<Config>): Promise<KeyStoreBase> {
        const cfg = config.normalize({
            ...maybeCfg || {},
        });
        const { storeName } = cfg;
        const store = idb.createStore(storeName);

        return new KeyStoreBase(cfg, store);
    }

    /* KeyStore Management */

    async blobExists(blobName: string): Promise<boolean> {
        const blob = await idb.getBlob(blobName, this.store);

        return blob !== null;
    }
    async deleteBlob(blobName: string): Promise<void> {
        return idb.rm(blobName, this.store);
    }
    async clear(): Promise<void> {
        return idb.clear();
    }
    async destroy(): Promise<void> {
        return idb.dropStore(this.store);
    }

    /* Cache Management */

    // Cache exported key material associated with the current session
    async cachePrivateKeyMaterial(
        privateKeyMaterial: PrivateKeyMaterial,
        sessionKey: string,
        sessionId: string,
        cfg?: Partial<Config>
    ): Promise<void> {
        const mergedCfg = config.merge(this.cfg, cfg);
        const salt = utils.randomBuf(DEFAULT_SALT_LENGTH);
        const key = await pbkdf2.deriveKey(
            sessionKey,
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
        const cachedKeyMaterial: CachedKeyMaterial = {
            encryptedPrivateKeyMaterial,
            sessionKeySalt: utils.arrBufToBase64(salt),
        };
        const cachedKeyMaterialStr = JSON.stringify(cachedKeyMaterial);
        const cachedKeyMaterialBlob = new Blob([cachedKeyMaterialStr], {
            type: 'application/json',
        });
        await idb.putBlob(sessionId, cachedKeyMaterialBlob, this.store);
    }

    // Retrieve cached key material associated with the current session
    async retrieveCachedPrivateKeyMaterial(
        sessionKey: string,
        sessionId: string,
        cfg?: Partial<Config>
    ): Promise<PrivateKeyMaterial> {
        const mergedCfg = config.merge(this.cfg, cfg);

        return idb.getBlob(sessionId, this.store).then((blob) => {
            if (blob === null) {
                throw new Error('Cached key material not found');
            }

            return blob.text().then((cachedKeyMaterialStr) => {
                const cachedKeyMaterial: CachedKeyMaterial = JSON.parse(
                    cachedKeyMaterialStr
                );
                const salt = utils.base64ToArrBuf(cachedKeyMaterial.sessionKeySalt);

                return pbkdf2.deriveKey(
                    sessionKey,
                    salt,
                    mergedCfg.hashAlg,
                    ['decrypt'],
                    config.symmKeyOpts(mergedCfg)
                ).then((key) => aes.decrypt(
                    cachedKeyMaterial.encryptedPrivateKeyMaterial,
                    key
                )).then((privateKeyMaterialStr) => JSON.parse(privateKeyMaterialStr)).catch((err) => {
                    throw new Error('Failed to decrypt cached key material');
                });
            });
        });
    }
}
