import { BrowserObject } from '@/app/types/bucket'

export const sortByType = (prev: BrowserObject, next: BrowserObject) => {
    const isPrevDir = prev.type === 'dir';
    const isNextDir = next.type === 'dir';

    if (isPrevDir && !isNextDir) {
        return -1;
    } else if (!isPrevDir && isNextDir) {
        return 1;
    } else {
        return 0;
    }
};

export const sortByName = (prev: BrowserObject, next: BrowserObject, reversed: boolean = false) => reversed ? prev.name.localeCompare(next.name) : next.name.localeCompare(prev.name);

export const sortByMetadataField = (prev: BrowserObject, next: BrowserObject, criteria: string, reversed: boolean = false) => reversed ? Number(prev.metadata[criteria]) - Number(next.metadata[criteria]) : Number(next.metadata[criteria]) - Number(prev.metadata[criteria]);

export const sortFiles = (prev: BrowserObject, next: BrowserObject, criteria: string, reversed: boolean = false) => {
    return criteria === 'name' ? sortByName(prev, next, reversed): sortByMetadataField(prev, next, criteria, reversed);
};