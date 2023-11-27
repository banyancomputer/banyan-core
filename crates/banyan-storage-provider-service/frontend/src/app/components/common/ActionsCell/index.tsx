import React, { ReactElement, useEffect, useRef, useState } from 'react';

import { popupClickHandler } from '@app/utils/clickHandlers';

import { Dots } from '@static/images';

export const ActionsCell: React.FC<{ actions: ReactElement; }> = ({ actions }) => {
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
            id="actionsCell"
            className="relative flex justify-end cursor-pointer"
            ref={actionsRef}
            onClick={toggleActionsVisibility}
        >
            <span className="pointer-events-none">
                <Dots />
            </span>
            {isActionsVisible &&
                <div className="absolute -top-4 right-6">
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
            }
        </div>
    );
};
