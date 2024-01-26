export function convertFileSize(bytes: number) {
    if(typeof bytes !== 'number') return;

    let size = bytes;
    let counter = 0;
    const BASE = 1024;
    const labels = ['B', 'KB', 'MB', 'GB', 'TB'];

    while(size >= BASE) {
        size /= BASE;
        counter++;
    };

    return `${+size.toFixed(2)} ${labels[counter]}`;
};

export function convertSubscriptionsSizes(rawSize: number) {
    if(typeof rawSize !== 'number') return;

    let size = rawSize;
    let counter = 0;
    const BASE = 1024;
    const labels = ['GB', 'TB'];

    while(size >= BASE) {
        size /= BASE;
        counter++;
    };

    return `${+size.toFixed(2)} ${labels[counter]}`;
};
