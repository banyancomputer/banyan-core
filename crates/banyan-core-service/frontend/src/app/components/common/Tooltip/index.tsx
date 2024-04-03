import React, { ReactElement, useEffect, useRef, useState } from 'react';

export const Tooltip: React.FC<{ className?: string, bodyClassName?: string, body: ReactElement, tooltip: ReactElement }> = ({ className, bodyClassName, body, tooltip }) => {
    const tooltipRef = useRef<null | HTMLDivElement>(null);
    const [tooltipPosition, setTooltipPosition] = useState({ x: 0, y: 0 });

    useEffect(() => {
        if (!tooltipRef) return;

        const listener = () => {
            const { x, y } = tooltipRef.current!.getBoundingClientRect();
            setTooltipPosition({ x, y });
        };

        tooltipRef.current?.addEventListener('mouseenter', listener);

        return () => {
            tooltipRef.current?.removeEventListener('mouseenter', listener);
        };
    }, [tooltipRef]);

    return (
        <div
            className={`relative group text-xxs text-bucket-actionsText ${className}`}
            ref={tooltipRef}
        >
            {body}
            <div
                className={`fixed top-[${tooltipPosition.y}px] left-[${tooltipPosition.x}px] hidden flex-col px-3 py-1 border-2 rounded-md border-border-regular bg-bucket-actionsBackground cursor-default whitespace-nowrap group-hover:flex z-10 ${bodyClassName} shadow-md`}
            >
                {tooltip}
            </div>
        </div>
    );
};
