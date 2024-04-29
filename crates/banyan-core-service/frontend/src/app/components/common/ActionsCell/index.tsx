import React, { ReactElement, useEffect, useRef, useState } from 'react';

import { popupClickHandler } from '@/app/utils';

import { Dots } from '@static/images/common';

export const ActionsCell: React.FC<{ actions: ReactElement; }> = ({ actions }) => {
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const actionsBodyRef = useRef<HTMLDivElement | null>(null);
    const [isActionsVisible, setIsActionsVisible] = useState(false);

    const toggleActionsVisibility = () => {
        const { y } = actionsRef.current!.getBoundingClientRect();
        const viewportHeight = window.innerHeight;
        const actionsHeight = actionsBodyRef.current!.scrollHeight;
        const overflow = (actionsHeight + y) - viewportHeight;
        const top = overflow > 0 ? y - overflow : y;
        actionsBodyRef.current!.style.top = `${top}px`;
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

    useEffect(() => {
        if (!isActionsVisible) return;

        const table = document.getElementById('table');
        if (!table) return;

        const listener = () => {
            setIsActionsVisible(false);
        };

        table.addEventListener('scroll', listener);

        return () => {
            document.removeEventListener('scroll', listener);
        };
    }, [isActionsVisible]);

    return (
        <div
            id="actionsCell"
            className="relative flex justify-center p-4 cursor-pointer"
            ref={actionsRef}
            onClick={toggleActionsVisibility}
        >
            <span className="pointer-events-none">
                <Dots />
            </span>
            <div
                ref={actionsBodyRef}
                className={`fixed z-20 transition-none ${isActionsVisible ? 'visible opacity-100' : 'invisible opacity-0'}`}
            >
                <div
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
