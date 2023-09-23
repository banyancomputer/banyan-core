import React, { useMemo } from 'react';
import { useRouter } from 'next/router';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { query } = useRouter();
    const queryCopy = { ...query };
    delete queryCopy.id

    const foldersPats = useMemo(() => Object.keys(queryCopy)[0]?.split('/') || [], [query]);

    return foldersPats;
};
