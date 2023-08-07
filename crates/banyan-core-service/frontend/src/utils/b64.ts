export function b64UrlEncode(base64str: string) {
    return base64str.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '.');
}

export function b64UrlDecode(base64str: string) {
    return base64str.replace(/-/g, '+').replace(/_/g, '/').replace(/\./g, '=');
}
