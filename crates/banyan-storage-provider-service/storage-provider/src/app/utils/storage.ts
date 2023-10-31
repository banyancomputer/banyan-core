export function convertFileSize(bytes: number, floatLength: number = 2) {
    if(typeof bytes !== 'number') return;

    let size = bytes;
    let counter = 0;
    const BASE = 1024;
    const labels = ['B', 'KB', 'MB', 'GB', 'TB'];

    while(size >= BASE) {
        size /= BASE;
        counter++;
    };

    return `${+size.toFixed(floatLength)} ${labels[counter]}`;
};
