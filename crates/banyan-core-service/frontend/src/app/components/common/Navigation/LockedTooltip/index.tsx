import React, { useEffect, useRef, useState } from 'react';

import { RequestBucketAccessModal } from '@components/common/Modal/RequestBucketAccessModal';

import { Bucket } from '@app/types/bucket';
import { useModal } from '@app/contexts/modals';
import { useAppSelector } from '@/app/store';

import { Lock } from '@static/images/buckets';

export const LockedTooltip: React.FC<{ bucket: Bucket, className?: string, size?: string, bodyClassName?: string }> = ({ bucket, className, size, bodyClassName }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.navigation.lockedTooltip);
    const tooltipRef = useRef<null | HTMLDivElement>(null);
    const [tooltipPosition, setTooltipPosition] = useState({ x: 0, y: 0 });
    const { openModal } = useModal();

    const stopPopagation = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        event.stopPropagation();
        event.preventDefault();
    };

    const requestAccess = () => {
        openModal(<RequestBucketAccessModal bucket={bucket} />);
    };

    useEffect(() => {
        if (!tooltipRef) return;

        const listener = () => {
            const { x, y } = tooltipRef.current!.getBoundingClientRect();
            setTooltipPosition({ x, y });
        }

        tooltipRef.current?.addEventListener('mouseenter', listener);

        return () => {
            tooltipRef.current?.removeEventListener('mouseenter', listener);
        };
    }, [tooltipRef]);

    return (
        <div
            className={`absolute group text-xxs text-bucket-actionsText ${className}`}
            ref={tooltipRef}
        >
            <Lock width={size || '20px'} height={size || '20px'} />
            <div
                className={`fixed top-[${tooltipPosition.y}px] left-[${tooltipPosition.x}px] max-w-[320px]  hidden flex-col px-3 py-1 border-2 rounded-md border-border-regular bg-bucket-actionsBackground whitespace-normal cursor-default group-hover:flex z-10 ${bodyClassName}`}
                onClick={stopPopagation}
            >
                {`${messages.youHaveNoAccess}`}
                {/* <span
                    className="font-semibold underline text-button-primary cursor-pointer"
                    onClick={requestAccess}
                    {`${messages.requestAccess} ${messages.here}`}
                </span> */}
            </div>
        </div>
    );
};
