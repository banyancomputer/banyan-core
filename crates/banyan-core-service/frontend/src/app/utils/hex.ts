export function stringToHex(string: string) {
    let hex = '';
    for(let i = 0; i < string.length; i++) {
        hex += string.charCodeAt(i).toString(16).padStart(2, '0');
    };

    return hex;
};

export function hexToString(hex: string) {
    let str = '';
    for (let i = 0; i < hex.length; i += 2) {
        let charCode = parseInt(hex.substr(i, 2), 16);
        str += String.fromCharCode(charCode);
    };

    return str;
}