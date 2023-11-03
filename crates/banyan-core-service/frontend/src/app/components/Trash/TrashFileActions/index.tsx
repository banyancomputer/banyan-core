import React, { ReactElement } from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from 'react-icons/fi';
import { PiArrowsLeftRight } from 'react-icons/pi';

import { MoveToModal } from '../../common/Modal/MoveToModal';
import { Action } from '../../common/FileActions';
import { useTomb } from '@/app/contexts/tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';


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
        new Action(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo),
        new Action(`${messages.remove}`, <FiTrash2 size="18px" />, remove),
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
