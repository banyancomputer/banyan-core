import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { BucketActions } from '@components/common/BucketActions';
import { LockedTooltip } from '@components/common/Navigation/LockedTooltip';

import { Bucket as IBucket } from '@/app/types/bucket';
import { popupClickHandler } from '@/app/utils';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { preventDefaultDragAction } from '@utils/dragHandlers';
import { useAppSelector } from '@/app/store';

import { BucketIcon } from '@static/images/buckets';
import { Dots, Question } from '@static/images/common';

export const Bucket: React.FC<{ bucket: IBucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.home.bucket);
    const { uploadFiles } = useFilesUpload();
    const bucketRef = useRef<HTMLDivElement | null>(null);
    const bucketActionsRef = useRef<HTMLDivElement | null>(null);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [isContextMenuVisible, setIsContextMenuVisible] = useState(false);
    const navigate = useNavigate();
    type messagesKeys = keyof typeof messages;

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
        if (!bucket.mount) return;
        // @ts-ignore
        if (event.target.id === 'bucketContextMenu') { return; }
        navigate(`/drive/${bucket.id}`);
    };

    const handleDrop = async (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);

        if (!event?.dataTransfer.files.length) { return; }

        try {
            await uploadFiles(event.dataTransfer.files, bucket, []);
        } catch (error: any) {
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, () => { });
        }
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
            className={`rounded-xl cursor-pointer transition-all bg-secondaryBackground border-1 border-border-regular ${!bucket.mount && 'cursor-not-allowed'}`}
            ref={bucketRef}
            onContextMenu={onContextMenu}
            onClick={openBucket}
            onDrop={handleDrop}
            onDragOver={preventDefaultDragAction}
        >
            <div
                className={`absolute ${!isContextMenuVisible && 'invisible'} transition-none z-10 cursor-pointer`}
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
            <div className="relative mb-4 flex justify-center py-24 bg-navigation-secondary rounded-t-xl z-0">
                <BucketIcon />
                <div
                    className="absolute right-2 top-2 p-1 cursor-pointer"
                    onClick={event => {
                        event.stopPropagation();
                        onContextMenu(event);
                    }}
                    id="bucketContextMenu"
                >
                    <span className="pointer-events-none">
                        <Dots />
                    </span>
                </div>
                {bucket.locked &&
                    <span className="absolute left-2 top-2 z-10 text-text-900"><LockedTooltip bucket={bucket} /></span>
                }
            </div>
            <div className="px-2 pb-2">
                <span className="mb-4 flex justify-between items-center text-ellipsis overflow-hidden whitespace-nowrap font-semibold">
                    {bucket.name}
                </span>
                <div className="flex flex-col gap-2 items-start text-xs font-normal">
                    <div className="flex items-center justify-between w-full">
                        <div className={`px-2 rounded-full text-mainBackground ${storageClassNames[bucket.storageClass]} capitalize`}>
                            {messages[bucket.storageClass as messagesKeys]}
                        </div>
                        <div className="text-text-400" title={messages[`${bucket.storageClass}Tooltip` as messagesKeys]}>
                            <Question width="24px" height="24px" />
                        </div>
                    </div>
                    <div className="capitalize">{bucket.bucketType}</div>
                    {bucket.snapshots.length ? <div>{bucket.snapshots.length} {`${messages.coldSnapshots}`}</div> : null}
                    <div className="flex justify-between items-center">
                        <div>{bucket.files.length} {`${messages.files}`}</div>
                    </div>
                </div>
            </div>
        </div>
    );
};
