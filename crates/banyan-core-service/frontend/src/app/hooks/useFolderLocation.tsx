import React, { useMemo } from 'react';
import { useLocation } from 'react-router-dom';

import { base64ToString } from '@utils/base64';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { search } = useLocation();

    const foldersPaths = useMemo(() => search ? search.slice(1).split('/').map(element => base64ToString(element)) : [], [search]);

    return foldersPaths;
};
