import { getDateLabel } from '@app/utils/date';

describe(
    'date',
    () => {
        test('getDateLabel', () => {
            expect(getDateLabel(1700062579)).toBe('Nov 15, 2023');
            expect(getDateLabel(1700062579, false)).toBe('Nov 15');
            expect(getDateLabel(0)).toBe('Jan 1, 1970');
        });
    }
);
