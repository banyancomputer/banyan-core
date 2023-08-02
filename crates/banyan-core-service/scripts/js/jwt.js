const jose = require("node-jose");
const fs = require('fs');

// TODO: we should be doing this in bash!

const args = process.argv.slice(2)
if (args.length != 4) {
  console.log("Usage: node index.js <pem_path> <key_id> <algorithm> <payload>")
  process.exit(1)
}

const pemPath = args[0]
const keyID = args[1]
const algorithm = args[2]
const payload = JSON.parse(args[3])

generateJWT(pemPath, keyID, algorithm, payload)
  .then((jwt) => {
    console.log(jwt)
  })

/**
 * Generate a JWT
 */
async function generateJWT(
  pemPath,
  keyID,
  algorithm,
  payload
) {
  const privatePem = fs.readFileSync(pemPath, 'utf8')
  const signingKey = await jose.JWK.asKey(
    privatePem,
    'pem'
  );

  const sign = await jose.JWS.createSign(
    { fields: { alg: algorithm, kid: keyID, typ: 'JWT' } },
    signingKey
  )
    .update(JSON.stringify(payload), 'utf8')
    .final();

  const signature = sign.signatures[0];
  return [signature.protected, sign.payload, signature.signature].join('.');
}
