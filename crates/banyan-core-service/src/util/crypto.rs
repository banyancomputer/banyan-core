pub mod ecdh;
pub mod ecdsa;
mod error;

pub use error::CryptoError;

// Javascript that needs to be reproduced in Rust...
//
// ```js
// var real_key_pair = await window.crypto.subtle.generateKey(
//     { name: 'ECDH', namedCurve: 'P-384' },
//     true,   // should always be extractable
//     ['deriveBits', 'deriveKey'],
// );
//
// var real_exported_private_key = await window.crypto.subtle.exportKey(
//     'pkcs8',
//     real_key_pair.privateKey,
// );
// var real_exported_public_key = await window.crypto.subtle.exportKey(
//     'spki',
//     real_key_pair.publicKey,
// );
//
// console.log('Exported Target Private Key: ' + btoa(new Uint8Array(real_exported_private_key)));
// // NDgsMTI5LDE4MiwyLDEsMCw0OCwxNiw2LDcsNDIsMTM0LDcyLDIwNiw2MSwyLDEsNiw1LDQzLDEyOSw0LDAsMzQsNCwxM
// // jksMTU4LDQ4LDEyOSwxNTUsMiwxLDEsNCw0OCw0NCwxMDgsODgsMjExLDksMTQ3LDE0OCwxMTIsMTY4LDUwLDgwLDEzMy
// // wyMzcsMTEyLDIyNCwxNTgsMzYsNTUsMTE5LDE5Niw0LDE5MiwyMzQsMjM0LDI3LDE2NiwxMTksMjUyLDEzMCw2MywxNDQ
// // sNTMsMTY1LDcsMjA5LDI1MSwxMDksMTM2LDQ1LDIwNSw4MiwyMDcsMjAxLDE5MSwxODIsMjI0LDgwLDEyMCwxNjEsMTAw
// // LDMsOTgsMCw0LDIyMiwzNSwxMzksMjUyLDU4LDE5LDEwMywzMSw0OCwxNjYsMjA5LDE3OCwxNzksMTg5LDIxMSwyNTUsM
// // jQ5LDEyNSwyMDAsNzIsNCwxNiwyMjYsNjAsMTg2LDIxNSwyMTcsMTY2LDg1LDU4LDE0MCw2LDI0MywxOTgsMjQyLDYzLD
// // IyNywyMzcsMTU5LDkwLDY0LDEyNSwyNTEsODMsMTgzLDEwMSwxNTcsMTY2LDE0NiwzNCwxMzIsMTE2LDIzMiw1OSwxMDM
// // sMTUyLDQ5LDI0NiwyMzEsMTA3LDQ2LDIxLDI4LDEzNCwyMjAsMTg1LDgxLDE2LDEyNiw4MSwxNDIsMTY5LDQxLDcxLDAs
// // MjQsMzcsMTIwLDE1OSwyMTMsMTU3LDE3MCwxNzYsMTY0LDIxNSw2LDE2OSwxODIsMTg4LDE2MiwxOTEsOTUsMjQwLDE5O
// // CwyMzIsMTg1
//
// console.log('Exported Target Public Key: ' + btoa(new Uint8Array(real_exported_public_key)));
// // NDgsMTE4LDQ4LDE2LDYsNyw0MiwxMzQsNzIsMjA2LDYxLDIsMSw2LDUsNDMsMTI5LDQsMCwzNCwzLDk4LDAsNCwyMjIsM
// // zUsMTM5LDI1Miw1OCwxOSwxMDMsMzEsNDgsMTY2LDIwOSwxNzgsMTc5LDE4OSwyMTEsMjU1LDI0OSwxMjUsMjAwLDcyLD
// // QsMTYsMjI2LDYwLDE4NiwyMTUsMjE3LDE2Niw4NSw1OCwxNDAsNiwyNDMsMTk4LDI0Miw2MywyMjcsMjM3LDE1OSw5MCw
// // 2NCwxMjUsMjUxLDgzLDE4MywxMDEsMTU3LDE2NiwxNDYsMzQsMTMyLDExNiwyMzIsNTksMTAzLDE1Miw0OSwyNDYsMjMx
// // LDEwNyw0NiwyMSwyOCwxMzQsMjIwLDE4NSw4MSwxNiwxMjYsODEsMTQyLDE2OSw0MSw3MSwwLDI0LDM3LDEyMCwxNTksM
// // jEzLDE1NywxNzAsMTc2LDE2NCwyMTUsNiwxNjksMTgyLDE4OCwxNjIsMTkxLDk1LDI0MCwxOTgsMjMyLDE4NQ==
// ```
//
// Let's pretend, real_key_pair is no longer available as we're another "client" that wants
// to encrypt the following new temporal key generated next. All we have available is the
// public key located at real_exported_public_key.
//
// ```js
// var temporal_key = await window.crypto.subtle.generateKey(
//     { name: 'AES-GCM', length: 256 },
//     true,   // should be false in prod
//     ['decrypt', 'encrypt'],
// );
//
// var exported_temporal_key = await window.crypto.subtle.exportKey(
//     'raw',
//     temporal_key,
// );
//
// console.log('Exported Temporal Key: ' + btoa(new Uint8Array(exported_temporal_key)));
// // MjIzLDE2NCwxMjksMjM0LDE2MywxMTEsOTcsMjU0LDEzNiwyMDQsODgsMjcsNDUsMTAyLDQ3LDE5Nyw3NiwyLDIyMywxN
// // DksNzMsMjI4LDI0LDEyNCwxNjYsMTgyLDgyLDkzLDE4Myw2MiwyMjksMTU=
// ```
//
// We have the new temporal key that needs to be encrypted and the public key of the target.
// Generate a fresh ephemeral key that will be discarded...
//
// ```js
// var ephemeral_key_pair = await window.crypto.subtle.generateKey(
//     { name: 'ECDH', namedCurve: 'P-384' },
//     true,   // should always be extractable
//     ['deriveBits', 'deriveKey'],
// );
//
// var ephemeral_exported_public_key = await window.crypto.subtle.exportKey(
//     'spki',
//     ephemeral_key_pair.publicKey,
// );
//
// console.log('Ephemeral Public Key: ' + btoa(new Uint8Array(ephemeral_exported_public_key)));
// // NDgsMTE4LDQ4LDE2LDYsNyw0MiwxMzQsNzIsMjA2LDYxLDIsMSw2LDUsNDMsMTI5LDQsMCwzNCwzLDk4LDAsNCw3NiwyN
// // ywxNjksMTUzLDIwMCwwLDcsMTg4LDc1LDE0NCwyMjEsMTM1LDEwNyw1MiwxNzcsMTk1LDYxLDE0OSw4NSwzMiwxOTQsMT
// // IsMTY2LDQzLDUzLDIxMSwxNDYsMTc4LDc1LDkzLDExNSwxNzUsMjM3LDg4LDE0Myw1MywyNCwyMzgsNTYsMjQsMTE4LDM
// // wLDIzNCwyMDQsODUsMjAxLDcwLDY4LDcwLDQ1LDEyMSwxOTEsMTIxLDI0OCw3NSwyNDMsMjQ5LDEyNSwxOTEsMTEyLDc4
// // LDEwMiw1LDE1NywxMTQsMjEwLDUzLDIwMSwxODksMTU0LDI0MCwxMTUsMjcsMTgxLDI0MiwxNTcsMTQzLDY0LDQ3LDIzM
// // ywxOTIsMTUyLDIxMiw5MCwxNzEsMjQwLDQ3LDI2LDE3MSwxNDYsODcsMjM1LDM4LDI1MywzLDkx
// ```
//
// Use it along with the public key to derive a common shared key that can be used to protect the
// temporal key.
//
// ```js
// var wrapping_key = await window.crypto.subtle.deriveKey(
//     { name: 'ECDH', public: real_key_pair.publicKey },
//     ephemeral_key_pair.privateKey,
//     { name: 'AES-KW', length: 256 },
//     true,   // should be false in prod
//     ['wrapKey', 'unwrapKey'],
// );
//
// var exported_wrapping_key = await window.crypto.subtle.exportKey(
//     'raw',
//     wrapping_key,
// );
//
// console.log('Exported Wrapping Key: ' + btoa(new Uint8Array(exported_wrapping_key)));
// // OSwyMjEsMTkwLDcxLDIwNSwxMTgsMTE2LDIxOCwyNSwxODcsMTUzLDI0OSwxODMsMjQ3LDEsMTc3LDcxLDE3MywxMDYsM
// // jI0LDIzNCwxNDYsMTk5LDE0OCwxNzEsODMsMTIxLDE3OSwyNDYsMTY4LDI0OCwxOTc
// ```
//
// Use the generated secret key to wrap the temporal key
//
// ```js
// let iv = await window.crypto.getRandomValues(new Uint8Array(12));
// let wrapped_temporal_key = await window.crypto.subtle.wrapKey(
//     'pkcs8', sampleTemporalKey, secretKey, 'AES-KW' },
// );
// ```
//
// We need to communicate iv, wrapped_temporal_key, and eph_public_key to the target so they can
// get back to the wrapping key and in turn unwrap the key... It is presumed that the target
// already has access to real_enc_key_pair.
//
// ```js
// let secretKey2 = await window.crypto.subtle.deriveKey(
//     { name: 'ECDH', public: eph_public_key },
//     real_enc_key_pair,
//     { name: 'AES-KW', length: 256 },
//     true,   // should be false in prod
//     ['wrapKey', 'unwrapKey'],
// );
// ```
//
// At this point secretKey should be equivalent to secretKey2
//
// ```js
// let unwrappedTemporalKey = await window.crypto.subtle.unwrapKey(
//     'pkcs8',
//     wrapped_temporal_key,
//     secretKey2,
//     'AES-KW',
//     { name: 'AES-GCM', length: 256 },
//     true,   // should be false in prod
//     ['decrypt', 'encrypt'],
// );
// ```
//
// And now `unwrappedTemporalKey` should be equivalent to `sampleTemporalKey` and can be used for
// encryption / decryption.
