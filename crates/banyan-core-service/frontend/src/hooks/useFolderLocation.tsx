import React, { useMemo } from 'react';
import { useRouter } from 'next/router';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { asPath } = useRouter();
    const foldersPats = useMemo(() => asPath.split('?')[1]?.split('/') || [], [asPath]);

    return foldersPats;
};
