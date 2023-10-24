export const publicPemWrap = (spki: string) => {
    // Wrap the public key in a pem
    const pemHeader = '-----BEGIN PUBLIC KEY-----\n';
    const pemFooter = '\n-----END PUBLIC KEY-----';

    // Break the spki into 64 character chunks
    const chunkedSpki = spki.match(/.{1,64}/g);
    if (!chunkedSpki) {
        throw new Error('Could not chunk spki');
    }

    const pem = pemHeader + chunkedSpki.join('\n') + pemFooter;

    return pem;
};

export const publicPemUnwrap = (pem: string) => {
    // Unwrap the public key from a pem
    const pemHeader = /-----BEGIN PUBLIC KEY-----\n/;
    const pemFooter = /\n-----END PUBLIC KEY-----/;
    const chunkedSpki = pem.replace(pemHeader, '').replace(pemFooter, '');
    const spki = chunkedSpki.replace(/\n/g, '');

    return spki;
};

export const privatePemWrap = (pkcs8: string) => {
    const pemHeader = '-----BEGIN PRIVATE KEY-----\n';
    const pemFooter = '\n-----END PRIVATE KEY-----';
    const chunkedPkcs8 = pkcs8.match(/.{1,64}/g);
    if (!chunkedPkcs8) {
        throw new Error('Could not chunk pkcs8');
    }
    const pem = pemHeader + chunkedPkcs8.join('\n') + pemFooter;

    return pem;
};

export const privatePemUnwrap = (pem: string) => {
    const pemHeader = /-----BEGIN PRIVATE KEY-----\n/;
    const pemFooter = /\n-----END PRIVATE KEY-----/;
    const chunkedPkcs8 = pem.replace(pemHeader, '').replace(pemFooter, '');
    const pkcs8 = chunkedPkcs8.replace(/\n/g, '');

    return pkcs8;
};
