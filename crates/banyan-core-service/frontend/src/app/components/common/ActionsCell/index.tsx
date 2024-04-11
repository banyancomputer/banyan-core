import React, { ReactElement, useEffect, useRef, useState } from 'react';

import { popupClickHandler } from '@/app/utils';

import { Dots } from '@static/images/common';

export const ActionsCell: React.FC<{ actions: ReactElement; }> = ({ actions }) => {
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const actionsBodyRef = useRef<HTMLDivElement | null>(null);
    const [isActionsVisible, setIsActionsVisible] = useState(false);

    const toggleActionsVisibility = () => {
        const table = document.getElementById('table');
        const tableHeight = table!.clientHeight;
        const scrollTop = table!.scrollTop;
        const actionsTop = actionsRef.current!.offsetTop;
        const actionsHeight = actionsBodyRef.current!.scrollHeight;

        const actionsOverflow = tableHeight - (actionsTop + actionsHeight + actionsRef.current!.clientHeight + 50 - scrollTop);

        if (actionsOverflow < 0) {
            actionsBodyRef.current!.style.top = `${actionsOverflow}px`;
        };

        setIsActionsVisible(prev => !prev);
    };

    useEffect(() => {
        if (!isActionsVisible) return;

        const listener = popupClickHandler(actionsRef.current!, setIsActionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [isActionsVisible]);

    return (
        <div
            id="actionsCell"
            className="relative flex justify-end cursor-pointer"
            ref={actionsRef}
            onClick={toggleActionsVisibility}
        >
            <span className="pointer-events-none">
                <Dots />
            </span>
            <div
                className={`right-0 top-6 z-10 transition-none ${isActionsVisible ? 'absolute visible opacity-100' : 'fixed invisible opacity-0'}`}
            >
                <div
                    ref={actionsBodyRef}
                    className="relative"
                    onClick={event => {
                        event.stopPropagation();
                        toggleActionsVisibility();
                    }}
                >
                    {actions}
                </div>
            </div>
        </div>
    );
};
