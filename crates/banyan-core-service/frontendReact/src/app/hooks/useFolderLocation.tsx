import React, { useMemo } from 'react';
import { useLocation } from 'react-router-dom';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { search } = useLocation();

    const foldersPaths = useMemo(() => search.split('/') || [], [search]);

    return foldersPaths;
};
