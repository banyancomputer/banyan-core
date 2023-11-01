import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';

import { Bucket as IBucket } from '@/lib/interfaces/bucket';
import { popupClickHandler } from '@/utils';

import { BucketActions } from '@/components/common/BucketActions';

import { BucketIcon } from '@static/images/buckets';
import { Dots, Question } from '@static/images/common';

export const Bucket: React.FC<{ bucket: IBucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const bucketRef = useRef<HTMLDivElement | null>(null);
    const bucketActionsRef = useRef<HTMLDivElement | null>(null);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [isContextMenuVisible, setIsContextMenuVisible] = useState(false);
    const { push } = useRouter();

    const onContextMenu = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        event.preventDefault();
        const bucketActionnBottom = bucketActionsRef.current!.clientHeight + event.clientY;
        const bucketActionsRight = bucketActionsRef.current!.clientWidth + event.clientX;
        const windowHeight = window.innerHeight;
        const windowWidth = window.innerWidth;

        const top = bucketActionnBottom <= windowHeight
            ?
            event.clientY
            :
            event.clientY - (bucketActionnBottom - windowHeight);

        const left = bucketActionsRight <= windowWidth
            ?
            event.clientX
            :
            event.clientX - (bucketActionsRight - windowWidth);

        setPosition({ x: left, y: top });
        setIsContextMenuVisible(true);
    };

    const openBucket = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        // @ts-ignore
        if (event.target.id === 'bucketContextMenu') { return; }
        push(`/bucket/${bucket.id}`);
    };

    useEffect(() => {
        const listener = popupClickHandler(bucketActionsRef.current!, setIsContextMenuVisible);
        const bucketListener = popupClickHandler(bucketRef.current!, setIsContextMenuVisible);
        document.addEventListener('click', listener);
        document.addEventListener('contextmenu', bucketListener);

        return () => {
            document.removeEventListener('click', listener);
            document.removeEventListener('contextmenu', bucketListener);
        };
    }, [bucketActionsRef]);

    const storageClassNames: Record<string, string> = {
        hot: 'bg-bucket-bucketClasshot',
        warm: 'bg-bucket-bucketClasswarm',
        cold: 'bg-bucket-bucketClasscold',
    };

    return (
        <div
            className="px-3 py-6 border-1 rounded-xl cursor-pointer transition-all bg-secondaryBackground hover:bg-bucket-bucketHoverBackground"
            ref={bucketRef}
            onContextMenu={onContextMenu}
            onClick={openBucket}
        >
            <div
                className={`absolute ${!isContextMenuVisible && 'invisible'} transition-none`}
                ref={bucketActionsRef}
                style={{ top: `${position.y}px`, left: `${position.x}px` }}
                id="bucketActions"
                onClick={event => {
                    event.stopPropagation();
                    setIsContextMenuVisible(false);
                }}
            >
                <BucketActions bucket={bucket} />
            </div>
            <h4 className="mb-3 flex items-center justify-between text-text-900 font-semibold">
                {bucket.name}
                <div
                    className="p-1 cursor-pointer"
                    onClick={event => {
                        event.stopPropagation();
                        onContextMenu(event);
                    }}
                    id="bucketContextMenu"
                >
                    <span className='pointer-events-none'>
                        <Dots />
                    </span>
                </div>
            </h4>
            <div
                className="mb-6 flex justify-center py-10 bg-bucket-bucketIconBackground rounded-xl"
            >
                <BucketIcon />
            </div>
            <div className="flex flex-col gap-2 items-start text-xs font-normal">
                <div className='flex items-center justify-between w-full'>
                    <div className={`px-2 rounded-full text-mainBackground ${storageClassNames[bucket.storageClass]} capitalize`}>
                        {`${messages[bucket.storageClass]}`}
                    </div>
                    <div className="text-text-400">
                        <Question width="24px" height="24px" />
                    </div>
                </div>
                <div className="capitalize">{bucket.bucketType}</div>
                {bucket.snapshots.length ? <div>{bucket.snapshots.length} {`${messages.coldSnapshots}`}</div> : null}
                <div className="flex justify-between items-center">
                    <div>{bucket.files.length} {`${messages.files}`}</div>
                    <div>{ }</div>
                </div>
            </div>
        </div>
    );
};
