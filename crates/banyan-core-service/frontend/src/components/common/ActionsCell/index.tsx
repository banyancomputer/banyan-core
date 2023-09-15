import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { HiDotsVertical } from 'react-icons/hi';

import { popupClickHandler } from '@/utils';

export const ActionsCell: React.FC<{ actions: ReactElement }> = ({ actions }) => {
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const [isActionsVisible, setIsActionsVisible] = useState(false);

    const toggleActionsVisibility = () => {
        setIsActionsVisible(prev => !prev);
    };

    useEffect(() => {
        const listener = popupClickHandler(actionsRef.current!, setIsActionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [actionsRef]);

    return (
        <div
            id='actionsCell'
            className="relative flex justify-end cursor-pointer"
            ref={actionsRef}
            onClick={toggleActionsVisibility}
        >
            <HiDotsVertical
                fill="#7f8ab0"
                size="20px"
                className="pointer-events-none"
            />

            {isActionsVisible &&
                <div className="absolute right-0 top-6">{actions}</div>
            }
        </div>
    );
};
