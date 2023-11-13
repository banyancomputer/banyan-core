import React, { ReactElement, useEffect, useRef, useState } from 'react';

import { Dots } from '@static/images/common';
import { popupClickHandler } from '@/app/utils';

export const ActionsCell: React.FC<{
    actions: ReactElement;
    offsetTop: number;
    tableRef: React.MutableRefObject<HTMLDivElement | null>;
}> = ({
    actions,
    offsetTop,
    tableRef,
}) => {
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const actionsBodyRef = useRef<HTMLDivElement | null>(null);
    const [isActionsVisible, setIsActionsVisible] = useState(false);
    const [overflow, setOverflow] = useState(0);

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

    useEffect(() => {
        setIsActionsVisible(false);
    }, [offsetTop]);

    useEffect(() => {
        if (!isActionsVisible) { return; }
        const rect = actionsBodyRef.current?.getBoundingClientRect();
        setOverflow(window.innerHeight - (rect?.bottom! - tableRef.current?.scrollTop! + 20));
    }, [actionsBodyRef, tableRef, isActionsVisible, offsetTop]);

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
                    <div
                        className="fixed right-14"
                        ref={actionsBodyRef}
                    >
                        <div
                            className="relative"
                            style={{ top: `-${offsetTop - Math.min(overflow, 0)}px` }}
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
