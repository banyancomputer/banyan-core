import React, { useMemo } from 'react';
import { useLocation } from 'react-router-dom';

import { hexToString } from '@utils/hex';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { search } = useLocation();

    const foldersPaths = useMemo(() => search ? search.slice(1).split('/').map(element => hexToString(element)) : [], [search]);

    return foldersPaths;
};
