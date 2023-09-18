import ECCKeyStore from '../ecc/keystore';
import config from '../config';
import IDB from '../idb';
import { Config, KeyStore } from '../types';

export async function init(maybeCfg?: Partial<Config>): Promise<KeyStore> {
  const cfg = config.normalize({
    ...(maybeCfg || {}),
  });

  return ECCKeyStore.init(cfg);
}

export async function clear(): Promise<void> {
  return IDB.clear();
}

export default {
  init,
  clear,
};
