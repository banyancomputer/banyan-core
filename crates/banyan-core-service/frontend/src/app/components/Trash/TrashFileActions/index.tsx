import React from 'react';
import { useIntl } from 'react-intl';

import { MoveToModal } from '@components/common/Modal/MoveToModal';
import { Action } from '@components/Bucket/BucketTable/FileActions';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';

import { MoveTo, Trash } from '@static/images/common';

export const TrashActions: React.FC<{ bucket: Bucket; file: BrowserObject }> = ({ bucket, file }) => {
    const { messages } = useIntl();
    const { } = useTomb();
    const { openModal } = useModal();

    const moveTo = () => {
        openModal(<MoveToModal bucket={bucket} file={file} parrentFolder={file} path={[]} />);
    };
    const makeCopy = async () => {
        try {

        } catch (error: any) { }
    };

    const remove = async () => {
        try {

        } catch (error: any) { }
    };

    const acrions = [
        new Action(`${messages.moveTo}`, <MoveTo width="18px" height="18px" />, moveTo),
        new Action(`${messages.remove}`, <Trash width="18px" height="18px" />, remove),
    ];

    return (
        <div className="relative w-48 text-xs font-medium bg-mainBackground rounded-xl shadow-md z-10 text-text-900">{
            acrions.map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={action.value}
                >
                    {action.icon} {action.label}
                </div>
            )
        }</div>
    );
};
