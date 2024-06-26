import { renderWithProviders } from '@/app/utils/tests/renderWithStore';
import { act, screen } from '@testing-library/react';
import Home from '@pages/home';
import { describe, expect, test } from 'vitest';
import { dispatch } from '@/app/store';
import { setBuckets } from '@/app/store/tomb/slice';
import { Bucket } from '@/app/types/bucket';

describe('Home page render tests', () => {

    test('Home page ', () => {
        renderWithProviders(<Home />);
        expect(screen.queryByTestId('bucket')).toBeNull();
        act(() => dispatch(setBuckets([{
            id: 'test',
            name: 'test',
            mount: null,
            bucketType: 'test',
            storageClass: 'test',
            isSnapshotValid: false,
            locked: false,
            snapshots: [],
            files: [],
            keys: []
        }])))
        expect(screen.queryByTestId('bucket')).toBeNull();
    })
})