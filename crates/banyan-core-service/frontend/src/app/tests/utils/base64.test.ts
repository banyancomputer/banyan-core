import { base64ToString, stringToBase64 } from '@utils/base64';

const decodedValue = 'test string';
/** Converted through https://www.base64encode.org */
const encodedValue = 'dGVzdCBzdHJpbmc=';

describe(
    'base64',
    () => {
        test('stringTOBase64', () => {
            expect(stringToBase64(decodedValue)).toBe(encodedValue);
        });
        test('base64ToString', () => {
            expect(base64ToString(encodedValue)).toBe(decodedValue);
        });
    }
);
