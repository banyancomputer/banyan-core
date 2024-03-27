export function stringToHex(string: string) {
    let hex, i;
    let result = "";
    for (i=0; i<string.length; i++) {
        hex = string.charCodeAt(i).toString(16);
        result += ("000"+hex).slice(-4);
    }

    return result
};

export function hexToString(string: string) {
    let j;
    let hexes = string.match(/.{1,4}/g) || [];
    let result = "";
    for(j = 0; j<hexes.length; j++) {
        result += String.fromCharCode(parseInt(hexes[j], 16));
    }

    return result;
};
