import { validateKeyphrase } from '@app/utils/validation';

describe(
    'validation',
    () => {
        test('validateKeyphrase', () => {
            const ERROR_MESSAGE = 'Not valid';
            const validator = validateKeyphrase(ERROR_MESSAGE);

            expect(validator('423445')).toBe(ERROR_MESSAGE);
            expect(validator('72947572')).toBe(undefined);
            expect(validator('testKeyphrase')).toBe(undefined);
        });
    }
);
