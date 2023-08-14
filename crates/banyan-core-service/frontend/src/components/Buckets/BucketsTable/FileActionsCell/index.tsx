import React, { useEffect, useRef, useState } from 'react';

import { BucketFile } from '@/lib/interfaces/bucket';
import { popupClickHandler } from '@/utils';

import { Dots } from '@static/images/common';

export const FileActionsCell: React.FC<{ file: BucketFile }> = ({ file }) => {
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const [isActionsVisible, setIsActionsVisible] = useState(false);

    useEffect(() => {
        const listener = popupClickHandler(actionsRef.current!, setIsActionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [actionsRef]);

    return (
        <div className="flex justify-end cursor-pointer" ref={actionsRef}><Dots /></div>
    );
};
