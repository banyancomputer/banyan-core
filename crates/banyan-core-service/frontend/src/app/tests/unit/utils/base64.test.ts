import { TextEncoder, TextDecoder } from 'util';
Object.assign(global, { TextDecoder, TextEncoder });
import { base64ToString, stringToBase64 } from '@utils/base64';
import { describe, expect, test } from 'vitest';

const decodedValue = 'test string';
/** Converted through https://www.base64encode.org */
const encodedValue = 'dGVzdCBzdHJpbmc=';

describe(
    'base64',
    () => {
        test('stringToBase64', () => {
            expect(stringToBase64(decodedValue)).toBe(encodedValue);
        });
        test('base64ToString', () => {
            expect(base64ToString(encodedValue)).toBe(decodedValue);
        });
    }
);
