# Banyan KeyStore

props to Fission for the [original implementation](
  https://github.com/fission-codes/keystore-idb
)

In-browser key management with IndexedDB and the Web Crypto API.

Securely store and use keys for encryption, decryption, and signatures. IndexedDB and Web Crypto keep keys safe from malicious javascript.

Supports only Elliptic Curves (P-384) for Asymmetric for Encryption and Decryption, Signing and Verification.

Implements escrowing Assymetric keys with passphrases.

Symmetric Encryption and Decryption is supported with AES-GCM.

Symmetric Key Wrapping is supported with AES-KW.

## Example Usage

```typescript
import * as Keystore from 'banyan-webcrypto/keystore'

async function run() {
  const ks = await Keystore.init()

  const msg = "Incididunt id ullamco et do."

  // TODO

  await ks.clear()
}

run()
```


## Development

```shell
# install dependencies
yarn

# run development server
yarn start

# build
yarn build

# test
# Note use nodeV16 when running tests
yarn test

# test w/ reloading
yarn test:watch
```
