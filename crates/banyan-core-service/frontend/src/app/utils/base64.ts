export function stringToBase64(string: string) {
    const utf8Encoder = new TextEncoder();
    const bytes = utf8Encoder.encode(string);
    const binString = Array.from(bytes, byte => String.fromCodePoint(byte)).join('');
    return btoa(binString);
};

export function base64ToString(base64String: string) {
    const binString = atob(base64String);
    const bytes = new Uint8Array(binString.length);
    for (let i = 0; i < binString.length; i++) {
        bytes[i] = binString.charCodeAt(i);
    }
    const utf8Decoder = new TextDecoder();
    return utf8Decoder.decode(bytes);
}