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
