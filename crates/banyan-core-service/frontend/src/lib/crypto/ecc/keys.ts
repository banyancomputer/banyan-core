import { webcrypto } from 'one-webcrypto';
import utils from '../utils';
import {
  ECC_EXCHANGE_ALG,
  ECC_WRITE_ALG,
} from '../constants';
import {
  EccCurve,
  KeyUse,
  PublicKey,
  ExportKeyFormat,
  SymmKey,
  PrivateKey,
} from '../types';
import { privatePemWrap, publicPemWrap, publicPemUnwrap } from '@/utils/pem';
import { checkValidKeyUse } from '../errors';

/**
 * Generate a new ECC key pair
 * @param curve The curve to use
 * @param use The use of the key pair, either exchange or write
 */
export async function genKeyPair(
  curve: EccCurve,
  use: KeyUse
): Promise<CryptoKeyPair> {
  checkValidKeyUse(use);
  const alg = use === KeyUse.Exchange ? ECC_EXCHANGE_ALG : ECC_WRITE_ALG;
  const uses: KeyUsage[] =
    use === KeyUse.Exchange ? ['deriveBits'] : ['sign', 'verify'];
  return webcrypto.subtle.generateKey(
    { name: alg, namedCurve: curve },
    true,
    uses
  );
}


/**
 * Import a public key from a base64 string
 * @param base64Key The base64 encoded public key
 * @param curve The curve to use
 * @param use The use of the key pair, either exchange or write
 */
export async function importPublicKeyPem(
  publicKeyPem: string,
  curve: EccCurve,
  use: KeyUse
): Promise<PublicKey> {
  checkValidKeyUse(use);
  const alg = use === KeyUse.Exchange ? ECC_EXCHANGE_ALG : ECC_WRITE_ALG;
  const uses: KeyUsage[] =
    use === KeyUse.Exchange ? ['deriveBits'] : ['verify'];
  let publicKey = publicPemUnwrap(publicKeyPem);
  const buf = utils.base64ToArrBuf(publicKey);
  return webcrypto.subtle.importKey(
    ExportKeyFormat.SPKI,
    buf,
    { name: alg, namedCurve: curve },
    true,
    uses
  );
}

/**
 * Export a public key to a base64 string
 * @param publicKey The public key to export
 */
export async function exportPublicKeyPem(publicKey: PublicKey): Promise<string> {
  const exp = await webcrypto.subtle.exportKey(
    ExportKeyFormat.SPKI,
    publicKey
  );
  let spki = utils.arrBufToBase64(exp);
  return publicPemWrap(spki)
}

/**
 * Export a private key to a base64 string
 * @param privateKey The private key to export
 */
export async function exportPrivateKeyPem(privateKey: PrivateKey): Promise<string> {
  const exp = await webcrypto.subtle.exportKey(
    ExportKeyFormat.PKCS8,
    privateKey
  );
  let pkcs8 = utils.arrBufToBase64(exp);
  return privatePemWrap(pkcs8)
}

export default {
  genKeyPair,
  importPublicKeyPem,
  exportPublicKeyPem,
  exportPrivateKeyPem,
};
