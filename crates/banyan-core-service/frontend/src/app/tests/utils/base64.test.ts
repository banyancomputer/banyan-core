import { hexToString, stringToHex } from '@utils/hex';

const decodedValue = 'test string';
/** Converted through https://www.base64encode.org */
const encodedValue = 'dGVzdCBzdHJpbmc=';

describe(
    'base64',
    () => {
        test('stringToHex', () => {
            expect(stringToHex(decodedValue)).toBe(encodedValue);
        });
        test('hexToString', () => {
            expect(hexToString(encodedValue)).toBe(decodedValue);
        });
    }
);
