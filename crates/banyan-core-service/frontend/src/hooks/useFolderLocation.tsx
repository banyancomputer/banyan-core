import React from 'react';
import { useRouter } from 'next/router';

/** Returns current folder nesting */
export const useFolderLocation = () => {
    const { asPath } = useRouter();

    return asPath.split("?")[1]?.split('/') || [];
}
