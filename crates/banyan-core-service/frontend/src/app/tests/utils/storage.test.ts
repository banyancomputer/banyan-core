import { convertFileSize } from "@app/utils/storage";

describe(
    'storage',
    () => {
        /** Checked through https://whatsabyte.com/P1/byteconverter.htm?utm_content=cmp-true */
        test('convertFileSize', () => {
            expect(convertFileSize(43000)).toBe('41.99 KB');
            expect(convertFileSize(36406700)).toBe('34.72 MB');
            expect(convertFileSize(31676406700)).toBe('29.5 GB');
            expect(convertFileSize(53691676406700)).toBe('48.83 TB');
            expect(convertFileSize(1000000000000000)).toBe('909.49 TB');
        });
    }
);