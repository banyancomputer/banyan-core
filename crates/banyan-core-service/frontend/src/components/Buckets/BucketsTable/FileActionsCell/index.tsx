import React, { useEffect, useRef, useState } from 'react';
import { HiDotsVertical } from "react-icons/hi"

import { BucketFile } from '@/lib/interfaces/bucket';
import { popupClickHandler } from '@/utils';

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
        <div className="flex justify-end cursor-pointer" ref={actionsRef}><HiDotsVertical fill='#7f8ab0' size="20px" /></div>
    );
};
