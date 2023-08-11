export function convertFileSize(bytes: number) {
    let size = bytes;
    let counter = 0;
    const BASE = 1024;
    const labels = ['b', 'kb', 'mb', 'gb', 'tb'];

    while(size >= BASE) {
        size /= BASE;
        counter++;
    };

    return `${+size.toFixed(2)} ${labels[counter]}`;
};
