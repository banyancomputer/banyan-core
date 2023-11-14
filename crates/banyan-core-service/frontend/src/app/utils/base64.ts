import { Buffer } from "buffer";

export function stringToBase64(string: string) {
    return Buffer.from(string, 'utf-8').toString('base64');
};

export function base64ToString(string: string) {
    return Buffer.from(string, 'base64').toString('utf-8');
};