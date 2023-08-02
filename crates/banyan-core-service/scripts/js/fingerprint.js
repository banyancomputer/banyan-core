const webcrypto = require('one-webcrypto').webcrypto;
const fs = require('fs');

// TODO: we should be doing this in bash!

/**
 * Fingerprint an ec public key -- does not generalize to curves other than P-384
 */
async function fingerprintEcPublicKey(
	publicKey
  ) {
	const publicKeyBytes = await webcrypto.subtle.exportKey(
        'raw',
		publicKey
	).then((key) => new Uint8Array(key));

	// TODO: Fix for other curves
	// NOTE: This makes it so that we can't use any curve other than P-384
	const size = 49;
	const compressedPoint = new Uint8Array(size);
	const x = publicKeyBytes.slice(1, size);
	const y = publicKeyBytes.slice(size);
  
	// Note:
	// first byte is 0x02 or 0x03 depending on the parity of the
	// y-coordinate, followed by the x coordinate. We can't technically
	// figure out whether the y-coodinate is odd without doing big number
	// arithmetic, but this is a fair approximation.
	compressedPoint[0] = y[y.length - 1] % 2 === 0 ? 0x02 : 0x03;
	compressedPoint.set(x, 1);
  
	const hash = await webcrypto.subtle.digest('SHA-1', compressedPoint);
	return new Uint8Array(hash);
  }

// Interpret a Uint8Array as a fingerprint
function prettyFingerprint(buf) {
	return Array.from(buf)
		.map((b) => b.toString(16).padStart(2, '0'))
		.join(':');
}

// Convert a base64 string to a Uint8Array
function base64ToArrBuf(base64) {
    const binary = atob(base64);
    const len = binary.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; ++i) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
}

const publicPemUnwrap = (pem) => {
	// Unwrap the public key from a pem
	const pemHeader = /-----BEGIN PUBLIC KEY-----\n/;
	const pemFooter = /\n-----END PUBLIC KEY-----/;
	const chunkedSpki = pem.replace(pemHeader, '').replace(pemFooter, '');
	const spki = chunkedSpki.replace(/\n/g, '');
	return spki;
};

const prettyFingerprintPem = async (
	pem
 ) => {
	const spki = publicPemUnwrap(pem);
    return webcrypto.subtle.importKey(
        'spki',
        base64ToArrBuf(spki),
        {
            name: 'ECDSA',
            namedCurve: 'P-384'
        },
        true,
        ['verify']
    ).then((key) => {
        return fingerprintEcPublicKey(key);
    }).then((fingerprint) => {
        return prettyFingerprint(fingerprint);
    });
};

const args = process.argv.slice(2)

if (args.length != 1) {
    console.log("Usage: node fingerprint.js <pem_path>")
    process.exit(1)
}

const pemPath = args[0]
fs.readFile(pemPath, 'utf8', (err, pem) => {
    if (err) {
        console.error(err)
        return
    }
    prettyFingerprintPem(pem)
        .then((fingerprint) => {
            console.log(fingerprint)
        })
})
    