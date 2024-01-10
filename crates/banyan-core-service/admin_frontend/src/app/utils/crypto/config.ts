import ecc from './ecc/keys';
import {
	ECC_EXCHANGE_ALG,
	ECC_WRITE_ALG,
	DEFAULT_ECC_CURVE,
	DEFAULT_SYMM_ALG,
	DEFAULT_SYMM_KEY_LENGTH,
	DEFAULT_HASH_ALG,
	DEFAULT_CHAR_SIZE,
	DEFAULT_STORE_NAME,
} from './constants';
import { Config, KeyUse, SymmKeyOpts } from './types';
import utils from './utils';

export const defaultConfig = {
	exchangeAlg: ECC_EXCHANGE_ALG,
	writeAlg: ECC_WRITE_ALG,
	curve: DEFAULT_ECC_CURVE,
	symmAlg: DEFAULT_SYMM_ALG,
	symmKeyLength: DEFAULT_SYMM_KEY_LENGTH,
	hashAlg: DEFAULT_HASH_ALG,
	charSize: DEFAULT_CHAR_SIZE,
	storeName: DEFAULT_STORE_NAME,
} as Config;

export function normalize(maybeCfg?: Partial<Config>): Config {
	let cfg;
	if (!maybeCfg) {
		cfg = defaultConfig;
	} else {
		cfg = {
			...defaultConfig,
			...maybeCfg,
		};
	}
	return cfg;
}

// Throws if ECC is not enabled
export async function eccEnabled(): Promise<boolean> {
	const keypair = await ecc.genKeyPair(DEFAULT_ECC_CURVE, KeyUse.Exchange);
	return true;
}

export function merge(cfg: Config, overwrites: Partial<Config> = {}): Config {
	return {
		...cfg,
		...overwrites,
	};
}

export function symmKeyOpts(cfg: Config): Partial<SymmKeyOpts> {
	return { alg: cfg.symmAlg, length: cfg.symmKeyLength };
}

export default {
	defaultConfig,
	normalize,
	eccEnabled,
	merge,
	symmKeyOpts,
};
